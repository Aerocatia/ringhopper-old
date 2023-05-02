use ringhopper::{error::ErrorMessageResult, types::Bounds, engines::h1::definitions::SoundFormat};
use libsamplerate_sys::*;

use vorbis_rs::*;

use std::{path::*, num::*, ffi::{CStr, c_long}, sync::{Arc, Mutex}};
use ringhopper::error::*;
use crate::file::*;
use ringhopper_proc::*;

use xbadpcm::*;

fn convert_pcm_16_to_f32(input: i16) -> f32 {
    match input {
        // Positive or zero
        i if i >= 0 => (i as f32) / (i16::MAX as f32),

        // Negative
        i => -(i as f32) / (i16::MIN as f32),
    }
}

fn convert_pcm_f32_to_16(input: f32) -> i16 {
    match input.clamp(-1.0, 1.0) {
        // Positive or zero
        i if i >= 0.0 => (i * (i16::MAX as f32)) as i16,

        // Negative
        i => -(i * (i16::MIN as f32)) as i16,
    }
}

use std::fmt::Display;

#[derive(Copy, Clone, PartialEq, Default)]
pub enum BitsPerSample {
    BPS(u32),

    #[default]
    Unknown
}
impl Display for BitsPerSample {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            BitsPerSample::BPS(n) => f.write_fmt(format_args!("{}-bit", n)),
            BitsPerSample::Unknown => f.write_str("N/A bps")
        }
    }
}

#[derive(Clone, Default)]
pub struct Sound {
    /// Imported name
    pub name: String,

    /// Imported path
    pub path: PathBuf,

    /// Sample rate in Hz
    pub sample_rate: u32,

    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: usize,

    /// Samples
    pub samples: Vec<i16>,

    /// Mouth data
    pub mouth_data: Vec<u8>,

    /// Encoded samples and their required buffer sizes.
    pub encoded_samples: Vec<(Vec<u8>, usize)>,

    /// Gain to set in tag data
    pub gain: f32,

    /// Skip fraction to set in tag data.
    pub skip_fraction: f32,

    /// Original sample rate in Hz
    pub original_sample_rate: u32,

    /// Original channel count (1 = mono, 2 = stereo)
    pub original_channels: usize,

    /// Original bits per sample
    pub original_bits_per_sample: BitsPerSample,

    /// Original codec
    pub original_codec: &'static str
}
impl Sound {
    fn new(path: PathBuf) -> ErrorMessageResult<Sound> {
        use symphonia::core::audio::SampleBuffer;
        use symphonia::core::codecs::DecoderOptions;
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;
        use symphonia::core::io::*;

        let mut path_copy = path.clone();
        path_copy.set_extension("");
        let name = path_copy.file_name()
                            .ok_or_else(|| ErrorMessage::AllocatedString(format!("Can't get \"{file}\"'s filename.", file=path.to_string_lossy())))?.to_str()
                            .ok_or_else(|| ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_non_utf8_path"),path=path.to_string_lossy())))?
                            .to_owned();

        let codecs = symphonia::default::get_codecs();
        let probe = symphonia::default::get_probe();
        let cursor = std::io::Cursor::new(read_file(&path)?);
        let stream = MediaSourceStream::new(Box::new(cursor), MediaSourceStreamOptions::default());

        let mut hint = Hint::new();
        let extension = path.extension().unwrap().to_str().unwrap().to_ascii_lowercase();
        hint.with_extension(&extension);

        let mut r = probe.format(&hint, stream, &FormatOptions::default(), &MetadataOptions::default())
                         .map_err(|e| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_cannot_decode"), file=path.to_string_lossy(), e=e.to_string())))?;

        let default_track = r.format.default_track().ok_or_else(|| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_cannot_decode_no_default_track"), file=path.to_string_lossy())))?.to_owned();
        let mut decoder = codecs.make(&default_track.codec_params, &DecoderOptions::default())
                                .map_err(|e| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_cannot_decode"), file=path.to_string_lossy(), e=e.to_string())))?;

        let channels = default_track.codec_params
                                    .channels
                                    .ok_or_else(|| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_cannot_get_sample_rate"), file=path.to_string_lossy())))?
                                    .count();

        let bps = default_track.codec_params
                               .bits_per_sample
                               .map(|f| BitsPerSample::BPS(f))
                               .unwrap_or(BitsPerSample::Unknown);

        if channels != 1 && channels != 2 {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_bad_channel_count"), file=path.to_string_lossy(), channels=channels)));
        }

        let sample_rate = default_track.codec_params.sample_rate
                                       .ok_or_else(|| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_cannot_get_sample_rate"), file=path.to_string_lossy())))?;
        if sample_rate == 0 {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_cannot_get_sample_rate"), file=path.to_string_lossy())));
        }

        let original_codec = codecs.get_codec(default_track.codec_params.codec).unwrap().short_name;

        let mut sample_buf = None;
        let mut samples = Vec::new();
        loop {
            let next_packet = r.format.next_packet();

            let packet = match next_packet {
                Ok(n) => n,
                Err(_) => break
            };

            if packet.track_id() != default_track.id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    if sample_buf.is_none() {
                        let spec = *audio_buf.spec();
                        let duration = audio_buf.capacity() as u64;
                        sample_buf = Some(SampleBuffer::<i16>::new(duration, spec));
                    }
                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(audio_buf);
                        samples.extend_from_slice(buf.samples());
                        buf.clear();
                    }
                }
                Err(symphonia::core::errors::Error::DecodeError(e)) => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_cannot_decode"), file=path.to_string_lossy(), e=e.to_string()))),
                Err(_) => break,
            }
        }

        if samples.is_empty() {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_cannot_decode_no_samples"), file=path.to_string_lossy())));
        }

        Ok(Sound { name, sample_rate, channels, samples, path, gain: 0.0, skip_fraction: 0.0, mouth_data: Vec::new(), original_channels: channels, original_sample_rate: sample_rate, original_bits_per_sample: bps, original_codec, encoded_samples: Vec::new() })
    }

    /// Encode the sound to the given format.
    pub fn encode(&mut self, format: SoundFormat, split: bool, compression_level: VorbisBitrateManagementStrategy, max_threads: usize) -> ErrorMessageResult<()> {
        let permutation_len = if split {
            232960 / 2 // number of bytes allowed max divided by 2 bytes per sample
        }
        else {
            self.samples.len()
        };

        // Nothing to encode? Ok.
        if permutation_len == 0 {
            self.encoded_samples.push((Vec::new(), 0));
            return Ok(());
        }

        let sample_count = self.samples.len();
        let total_chunk_count = self.samples.len() / permutation_len;
        let sample_rate = NonZeroU32::new(self.sample_rate).unwrap();
        let channels = NonZeroU8::new(self.channels as u8).unwrap();

        if max_threads <= 1 {
            self.encoded_samples.reserve_exact(total_chunk_count);
            for p in (0..sample_count).step_by(permutation_len) {
                let start = p;
                let end = (p.saturating_add(permutation_len)).min(sample_count);
                self.encoded_samples.push(encode_block(&self.samples[start..end], channels, sample_rate, format, compression_level, &self.name)?);
            }
        }
        else {
            let mut threads = Vec::with_capacity(max_threads);
            let encoded_samples = Arc::new(Mutex::new(Vec::with_capacity(total_chunk_count)));
            for _ in 0..max_threads {
                let samples = self.samples.clone();
                let permutation_name = self.name.clone();
                let channels = channels.clone();
                let sample_rate = sample_rate.clone();
                let format = format.clone();
                let compression_level = compression_level.clone();
                let permutation_len = permutation_len.clone();
                let encoded_samples = encoded_samples.clone();
                let sample_count = sample_count.clone();

                threads.push(std::thread::spawn(move || -> ErrorMessageResult<()> {
                    let mut index = 0;
                    for (start, index) in (0..sample_count).step_by(permutation_len).map(|p| { index = index + 1; (p, index - 1) }) {
                        let end = (start.saturating_add(permutation_len)).min(sample_count);

                        // Reserve our spot!
                        let mut es = encoded_samples.lock().unwrap();
                        if es.len() != index {
                            continue;
                        }
                        es.push((Vec::new(), 0)); // push a default value as a placeholder
                        drop(es);

                        // Put the real value in now
                        let samples = encode_block(&samples[start..end], channels, sample_rate, format, compression_level, &permutation_name)?;
                        encoded_samples.lock().unwrap()[index] = samples;
                    }
                    Ok(())
                }))
            }
            for t in threads {
                t.join().unwrap()?;
            }
            self.encoded_samples.append(&mut encoded_samples.lock().unwrap());
        }

        Ok(())
    }

    /// Generate mouth data.
    pub fn generate_mouth_data(&mut self) {
        // Basically, take the sample rate, multiply by channel count, divide by tick rate (30 Hz), and round the result
        let samples_per_tick = ((self.sample_rate as usize * self.channels as usize) as f64 / 30.0).round() as usize;
        let sample_count = self.samples.len();

        // Generate samples, adding an extra tick for incomplete ticks
        let tick_count = (sample_count + samples_per_tick - 1) / samples_per_tick;
        let mut mouth_data_floats = Vec::with_capacity(tick_count);
        let mut max: f32 = 0.0;
        let mut min: f32 = 1.0;
        let mut total: f32 = 0.0;
        for i in (0..self.samples.len()).step_by(samples_per_tick) {
            let tick_start = i;
            let tick_end = i.saturating_add(samples_per_tick).min(self.samples.len());
            let mut tick_total = 0.0;
            for i in tick_start..tick_end {
                let s = convert_pcm_16_to_f32(self.samples[i]).abs();
                tick_total += s;
            }

            let s = tick_total / ((tick_end - tick_start) as f32);
            min = min.min(s);
            max = max.max(s);
            total += s;
            mouth_data_floats.push(s);
        }

        let max = max;
        let min = min;
        let average = total / (mouth_data_floats.len() as f32);
        let range = (max + average) / 2.0 - min;

        // Do nothing if there's no range
        if range == 0.0 {
            self.mouth_data = mouth_data_floats.into_iter().map(|f| (f * (u8::MAX as f32) + 0.5) as u8).collect();
        }

        // Otherwise write it
        else {
            self.mouth_data = mouth_data_floats.into_iter().map(|f| (((f - min) / range).clamp(0.0, 1.0) * (u8::MAX as f32) + 0.5) as u8).collect();
        }
    }
}

#[derive(Clone)]
pub struct PitchRange {
    /// Name of the pitch range
    pub name: String,

    /// Import path of the pitch range
    pub path: PathBuf,

    /// Permutations of the pitch range
    pub permutations: Vec<Sound>,

    /// Default pitch to set in the tag
    pub natural_pitch: f32,

    /// Default pitch bounds to set in the tag.
    pub pitch_bounds: Bounds<f32>,
}

impl PitchRange {
    fn new(name: String, path: PathBuf, mut input_files: Vec<PathBuf>) -> ErrorMessageResult<PitchRange> {
        // Filter out files that are not supported as input as well as non-files
        input_files.retain(|f|
            if !f.is_file() {
                false
            }
            else if let Some(n) = f.extension() {
                match n.to_str() {
                    None => false,
                    Some(n) => n.eq_ignore_ascii_case("wav") || n.eq_ignore_ascii_case("flac")
                }
            }
            else {
                false
            }
        );
        if input_files.is_empty() {
            return Err(ErrorMessage::AllocatedString(format!("Pitch range \"{pitch_range}\"'s directory \"{pitch_range_dir}\" does not contain any importable files.", pitch_range=name, pitch_range_dir=path.to_string_lossy())));
        }

        // Add all permutations
        let mut permutations = Vec::with_capacity(input_files.len());
        for i in input_files {
            permutations.push(Sound::new(i)?);
        }
        Ok(PitchRange { name, path, permutations, natural_pitch: f32::default(), pitch_bounds: Bounds::default() })
    }
}

/// Resample the given samples to the new sample rate.
pub fn resample(samples: &[i16], channel_count: usize, ratio: f64) -> ErrorMessageResult<Vec<i16>> {
    // Calculate the ratio
    let old_frame_count = samples.len() / channel_count;
    let new_frame_count = (old_frame_count as f64 * (ratio + 0.95)).round() as usize; // allocate a few extra samples to make sure it works

    // Convert float to 16-bit PCM
    let mut input_samples = Vec::with_capacity(samples.len());
    for i in samples {
        input_samples.push(convert_pcm_16_to_f32(*i));
    }

    // Do a checked_mul so if we overflow, we panic here rather then do UB
    let mut output_samples = Vec::with_capacity(new_frame_count.checked_mul(channel_count).unwrap());

    // Instantiate our resampler
    let mut res = 0i32;
    let state = unsafe { src_new(SRC_SINC_BEST_QUALITY as std::ffi::c_int, channel_count as std::ffi::c_int, &mut res) };
    if res != 0 {
        panic!("Failed to resample! src_strerror: {}", unsafe { CStr::from_ptr(src_strerror(res) as *const i8).to_str().unwrap() });
    }

    // Initialize our input struct
    let mut data = SRC_DATA::default();
    data.src_ratio = ratio;
    data.data_in = input_samples.as_ptr();
    data.data_out = output_samples.as_mut_ptr();

    // Loop on all samples
    let samples_per_go = old_frame_count.min(10000);
    let mut frames_processed = 0usize;
    let mut frames_output = 0usize;
    loop {
        let samples_to_process = (old_frame_count - frames_processed).min(samples_per_go);
        let ending = data.end_of_input == 1;

        data.input_frames_used = 0;
        data.data_in = input_samples[frames_processed * channel_count..].as_ptr();
        data.data_out = unsafe { output_samples.as_mut_ptr().add(frames_output * channel_count) };
        data.input_frames = samples_to_process as c_long;
        data.output_frames = (c_long::MAX as usize).min(new_frame_count - frames_output) as c_long;

        // Process
        res = unsafe { src_process(state, &mut data) };
        if res != 0 {
            unsafe { src_delete(state) };
            panic!("Failed to resample! src_strerror: {}", unsafe { CStr::from_ptr(src_strerror(res) as *const i8).to_str().unwrap() });
        }

        // Increment
        let output_frames = data.output_frames_gen as usize;
        frames_output += output_frames;
        frames_processed += data.input_frames_used as usize;

        // If we are done, break
        if ending {
            break;
        }

        // If we did not process any samples, signal that we are finishing
        if data.input_frames_used == 0 {
            data.end_of_input = 1;
        }
    }

    // We're done. Close secret rabbit code now.
    unsafe { src_delete(state); }

    // Check if we blew it.
    debug_assert_eq!(old_frame_count, frames_processed, "Did not process all frames! ({} expected, got {})", old_frame_count, frames_processed);

    // Resize the buffer to whatever our new size is
    unsafe { output_samples.set_len(frames_output * channel_count) };

    // Create a new buffer containing our new samples
    let mut new_samples = Vec::with_capacity(output_samples.len());
    for s in output_samples {
        new_samples.push(convert_pcm_f32_to_16(s));
    }

    // Return
    Ok(new_samples)
}

/// Change the channel count of the given samples.
pub fn remix(samples: &[i16], old_channel_count: usize, new_channel_count: usize) -> Vec<i16> {
    let mut new_samples = Vec::with_capacity(samples.len() / old_channel_count * new_channel_count);
    if old_channel_count == 1 && new_channel_count == 2 {
        for s in samples {
            new_samples.push(*s);
            new_samples.push(*s);
        }
    }
    else if old_channel_count == 2 && new_channel_count == 1 {
        for s in (0..samples.len()).step_by(2) {
            let avg = ((samples[s] as i32) + (samples[s+1] as i32)) / 2;
            new_samples.push(avg as i16);
        }
    }
    else {
        panic!("Can't convert channel count from {} to {}", old_channel_count, new_channel_count);
    }
    new_samples
}

/// Load the contents of the sound tag from the data directory
pub fn load_data_dir(data: &Path) -> ErrorMessageResult<Vec<PitchRange>> {
    // Get the directory.
    let get_contents_from_dir = |directory: &Path| -> ErrorMessageResult<Vec<PathBuf>> {
        let mut files = Vec::new();
        for d in std::fs::read_dir(&directory).map_err(|e| ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_iterating_directory"), path=directory.to_string_lossy(), error=e)))? {
            let file = d.map_err(|e| ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_iterating_directory"), path=directory.to_string_lossy(), error=e)))?;
            files.push(file.path());
        }
        files.sort_by(|a,b| a.file_name().cmp(&b.file_name())); // sort the contents alphabetically
        Ok(files)
    };
    let contents = get_contents_from_dir(&data)?;
    let mut contains_dirs = false;
    let mut contains_files = false;
    for i in &contents {
        if i.is_dir() {
            contains_dirs = true;
        }
        else {
            contains_files = true;
        }
    }
    if contains_dirs && contains_files {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_bad_directory_mixed_files_dirs"), dir=data.to_string_lossy())));
    }
    if !contains_dirs && !contains_files {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_bad_directory_empty"), dir=data.to_string_lossy())));
    }

    // Populate the pitch ranges.
    let mut pitch_ranges = Vec::new();
    if contains_files {
        pitch_ranges.push(PitchRange::new("default".to_owned(), data.to_owned(), contents)?);
    }
    else if contains_dirs {
        for i in contents {
            let filename = i.file_name()
                            .ok_or_else(|| ErrorMessage::AllocatedString(format!("Can't get \"{file}\"'s filename.", file=i.to_string_lossy())))?.to_str()
                            .ok_or_else(|| ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_non_utf8_path"),path=i.to_string_lossy())))?
                            .to_owned();
            pitch_ranges.push(PitchRange::new(filename, i.clone(), get_contents_from_dir(&i)?)?);
        }
    }
    else {
        unreachable!()
    }

    Ok(pitch_ranges)
}

/// Get the highest sample rate and channel count, respectively, of all pitch ranges.
pub fn get_best_sample_rate_and_channel_count(pitch_ranges: &[PitchRange]) -> ErrorMessageResult<(u32, usize)> {
    let mut highest_channel_count = 0;
    let mut highest_sample_rate = 0;
    for pi in 0..pitch_ranges.len() {
        let p = &pitch_ranges[pi];

        // If we have multiple pitch ranges of the same name, bad.
        for pj in 0..pi {
            let p_other = &pitch_ranges[pj];
            if p_other.name.eq_ignore_ascii_case(&p.name) {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_ambiguous_pitch_range"),
                                                                 pitch_range=p_other.name,
                                                                 pitch_range_1=p_other.path.to_string_lossy(),
                                                                 pitch_range_2=p.path.to_string_lossy())));
            }
        }

        // If we have multiple sound permutations of the same name, bad.
        for i in 0..p.permutations.len() {
            for j in 0..i {
                let a = &p.permutations[j];
                let b = &p.permutations[i];
                if a.name.eq_ignore_ascii_case(&b.name) {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_ambiguous_permutation"),
                                                                     permutation=a.name,
                                                                     pitch_range=p.name,
                                                                     permutation_1=p.permutations[j].path.to_string_lossy(),
                                                                     permutation_2=p.permutations[i].path.to_string_lossy())));
                }
            }
        }

        // Next, get the channel count/sample rate
        for permutation in &p.permutations {
            highest_sample_rate = highest_sample_rate.max(permutation.sample_rate);
            highest_channel_count = highest_channel_count.max(permutation.channels);
        }
    }

    Ok((highest_sample_rate, highest_channel_count))
}


fn encode_block(samples_to_encode: &[i16], channels: NonZeroU8, sample_rate: NonZeroU32, format: SoundFormat, compression_level: VorbisBitrateManagementStrategy, permutation: &str) -> ErrorMessageResult<(Vec<u8>, usize)> {
    const UNCOMPRESSED_LIMIT: usize = u32::MAX as usize;
    let buffer_size = samples_to_encode.len() * 2;

    match format {
        SoundFormat::Pcm => {
            let mut v = Vec::with_capacity(buffer_size);
            for i in samples_to_encode {
                v.extend_from_slice(&i.to_be_bytes());
            }
            if buffer_size > UNCOMPRESSED_LIMIT {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_permutation_size_exceeded"),
                                                         permutation=permutation,
                                                         size=buffer_size,
                                                         limit=UNCOMPRESSED_LIMIT)))
            }

            return Ok((v, buffer_size));
        }
        SoundFormat::OggVorbis => {
            let mut v = Vec::with_capacity(buffer_size);

            // Work around a segfault with the encoder by only encoding this many samples at once.
            const MAX_SPLIT_SIZE: usize = 1024;

            let total_frame_count = samples_to_encode.len() / (channels.get() as usize);

            // Encode
            (|| -> Result<(), VorbisError> {
                let mut encoder = VorbisEncoder::new(
                    0,
                    [("ENCODER", env!("invader_version"))],
                    sample_rate,
                    channels,
                    compression_level,
                    None,
                    &mut v
                )?;

                match channels.get() {
                    1 => {
                        let samples_mono: Vec<f32> = samples_to_encode.iter().map(|i| convert_pcm_16_to_f32(*i)).collect();
                        debug_assert_eq!(total_frame_count, samples_mono.len());

                        for i in (0..total_frame_count).step_by(MAX_SPLIT_SIZE) {
                            let start = i;
                            let end = i.saturating_add(MAX_SPLIT_SIZE).min(total_frame_count);
                            encoder.encode_audio_block(&[&samples_mono[start..end]])?;
                        }
                    },
                    2 => {
                        let l: Vec<f32> = samples_to_encode.iter().step_by(2).map(|i| convert_pcm_16_to_f32(*i)).collect();
                        let r: Vec<f32> = samples_to_encode.iter().skip(1).step_by(2).map(|i| convert_pcm_16_to_f32(*i)).collect();

                        debug_assert_eq!(total_frame_count, l.len());
                        debug_assert_eq!(total_frame_count, r.len());

                        for i in (0..total_frame_count).step_by(MAX_SPLIT_SIZE) {
                            let start = i;
                            let end = i.saturating_add(MAX_SPLIT_SIZE).min(total_frame_count);
                            encoder.encode_audio_block(&[&l[start..end], &r[start..end]])?;
                        }
                    },
                    _ => panic!()
                }

                encoder.finish().map(|_| ())
            })().map_err(|e| ErrorMessage::AllocatedString(e.to_string()))?;

            if buffer_size > UNCOMPRESSED_LIMIT {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_permutation_size_exceeded"),
                                                         permutation=permutation,
                                                         size=buffer_size,
                                                         limit=UNCOMPRESSED_LIMIT)))
            }

            return Ok((v, buffer_size));
        }

        SoundFormat::XboxAdpcm => {
            let mut v = Vec::new();

            // Encode
            (|| -> Result<(), ()> {
                let mut encoder = XboxADPCMEncoder::new(channels.get() as usize, 3, &mut v);
                match channels.get() {
                    1 => {
                        encoder.encode(&[&samples_to_encode])?;
                    },
                    2 => {
                        let total_frame_count = samples_to_encode.len() / channels.get() as usize;

                        let l: Vec<i16> = samples_to_encode.iter().step_by(2).map(|i| *i).collect();
                        let r: Vec<i16> = samples_to_encode.iter().skip(1).step_by(2).map(|i| *i).collect();

                        debug_assert_eq!(total_frame_count, l.len());
                        debug_assert_eq!(total_frame_count, r.len());

                        encoder.encode(&[&l, &r])?;
                    },
                    _ => panic!()
                }
                encoder.finish()
            })().unwrap();

            return Ok((v, 0));
        }

        SoundFormat::ImaAdpcm => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.sound.error_ima_adpcm_not_supported"))),
    }
}
