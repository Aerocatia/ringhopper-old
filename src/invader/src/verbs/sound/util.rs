use ringhopper::{error::ErrorMessageResult, types::Bounds, engines::h1::definitions::SoundFormat};
use rubato::{Resampler, SincFixedIn, InterpolationType, InterpolationParameters, WindowFunction};

use vorbis_rs::*;

use std::{path::*, num::*};
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

#[derive(Copy, Clone, PartialEq)]
pub enum BitsPerSample {
    BPS(u32),
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

#[derive(Clone)]
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
        hint.with_extension(path.extension().unwrap().to_str().unwrap());

        let mut r = probe.format(&hint, stream, &FormatOptions::default(), &MetadataOptions::default())
                         .map_err(|e| ErrorMessage::AllocatedString(format!("Can't decode \"{file}\": {e}", file=path.to_string_lossy(), e=e.to_string())))?;

        let default_track = r.format.default_track().ok_or_else(|| ErrorMessage::AllocatedString(format!("File \"{file}\" has no default track", file=path.to_string_lossy())))?.to_owned();
        let mut decoder = codecs.make(&default_track.codec_params, &DecoderOptions::default())
                                .map_err(|e| ErrorMessage::AllocatedString(format!("Can't decode \"{file}\": {e}", file=path.to_string_lossy(), e=e.to_string())))?;

        let channels = default_track.codec_params
                                    .channels
                                    .ok_or_else(|| ErrorMessage::AllocatedString(format!("Can't get sample rate from \"{file}\"", file=path.to_string_lossy())))?
                                    .count();

        let bps = default_track.codec_params
                               .bits_per_sample
                               .map(|f| BitsPerSample::BPS(f))
                               .unwrap_or(BitsPerSample::Unknown);

        if channels != 1 && channels != 2 {
            return Err(ErrorMessage::AllocatedString(format!("Expected 1 or 2 channels. Found {channels} in \"{file}\"", file=path.to_string_lossy(), channels=channels)));
        }

        let sample_rate = default_track.codec_params.sample_rate
                                       .ok_or_else(|| ErrorMessage::AllocatedString(format!("Can't get sample rate from \"{file}\"", file=path.to_string_lossy())))?;
        if sample_rate == 0 {
            return Err(ErrorMessage::AllocatedString(format!("Invalid sample rate {sample_rate} Hz from \"{file}\"", file=path.to_string_lossy(), sample_rate=sample_rate)));
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
                Err(symphonia::core::errors::Error::DecodeError(e)) => return Err(ErrorMessage::AllocatedString(format!("Can't decode \"{file}\": {e}", file=path.to_string_lossy(), e=e.to_string()))),
                Err(_) => break,
            }
        }

        Ok(Sound { name, sample_rate, channels, samples, path, gain: 0.0, skip_fraction: 0.0, mouth_data: Vec::new(), original_channels: channels, original_sample_rate: sample_rate, original_bits_per_sample: bps, original_codec, encoded_samples: Vec::new() })
    }

    /// Encode the sound to the given format.
    pub fn encode(&mut self, format: SoundFormat, split: bool, compression_level: VorbisBitrateManagementStrategy) -> ErrorMessageResult<()> {
        let permutation_len = if split {
            116480
        }
        else {
            self.samples.len()
        };

        let sample_rate = NonZeroU32::new(self.sample_rate).unwrap();
        let channels = NonZeroU8::new(self.channels as u8).unwrap();

        const UNCOMPRESSED_LIMIT: usize = u32::MAX as usize;

        for p in (0..self.samples.len()).step_by(permutation_len) {
            let start = p;
            let end = (p.saturating_add(permutation_len)).min(self.samples.len());
            let samples_to_encode = &self.samples[start..end];
            let buffer_size = samples_to_encode.len() * 2;

            match format {
                SoundFormat::_16bitPcm => {
                    let mut v = Vec::with_capacity(buffer_size);
                    for i in samples_to_encode {
                        v.extend_from_slice(&i.to_be_bytes());
                    }
                    if buffer_size > UNCOMPRESSED_LIMIT {
                        return Err(ErrorMessage::AllocatedString(format!("Permutation \"{permutation}\"'s uncompressed size exceeds the maximum size ({size} > {limit})",
                                                                 permutation=self.name,
                                                                 size=buffer_size,
                                                                 limit=UNCOMPRESSED_LIMIT)))
                    }

                    self.encoded_samples.push((v, buffer_size));
                }
                SoundFormat::OggVorbis => {
                    let mut v = Vec::with_capacity(buffer_size);

                    // Work around a segfault with the encoder by only encoding this many samples at once.
                    const MAX_SPLIT_SIZE: usize = 1024;

                    let total_frame_count = samples_to_encode.len() / self.channels;

                    // Encode
                    (|| -> Result<(), VorbisError> {
                        let mut encoder = VorbisEncoder::new(
                            p as i32,
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

                        encoder.finish()
                    })().map_err(|e| ErrorMessage::AllocatedString(e.to_string()))?;

                    if buffer_size > UNCOMPRESSED_LIMIT {
                        return Err(ErrorMessage::AllocatedString(format!("Permutation \"{permutation}\"'s uncompressed size exceeds the maximum size ({size} > {limit})",
                                                                 permutation=self.name,
                                                                 size=buffer_size,
                                                                 limit=UNCOMPRESSED_LIMIT)))
                    }

                    self.encoded_samples.push((v, buffer_size));
                }

                SoundFormat::XboxAdpcm => {
                    let mut v = Vec::new();

                    // Encode
                    (|| -> Result<(), ()> {
                        let mut encoder = XboxADPCMEncoder::new(self.channels, 3, &mut v);
                        match channels.get() {
                            1 => {
                                encoder.encode(&[&samples_to_encode])?;
                            },
                            2 => {
                                let total_frame_count = samples_to_encode.len() / self.channels;

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

                    self.encoded_samples.push((v, 0));
                }

                SoundFormat::ImaAdpcm => return Err(ErrorMessage::StaticString("IMA ADPCM is not supported")),
            }
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
    pub pitch_bounds: Bounds<f32>
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
                    Some("wav") => true,
                    Some("flac") => true,
                    _ => false
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
pub fn resample(samples: &[i16], channel_count: usize, old_sample_rate: u32, new_sample_rate: u32) -> Vec<i16> {
    let real_sample_count = samples.len() / channel_count;

    let params = InterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: InterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };
    let mut resampler = SincFixedIn::<f32>::new(
        new_sample_rate as f64 / old_sample_rate as f64,
        2.0,
        params,
        real_sample_count,
        channel_count
    ).unwrap();

    let mut new_samples = Vec::new();
    match channel_count {
        1 => {
            let mut input = Vec::with_capacity(real_sample_count);
            for i in samples {
                input.push(convert_pcm_16_to_f32(*i));
            }
            let result = resampler.process(&[input], None).unwrap();
            new_samples.reserve(result.len());
            for i in &result[0] {
                new_samples.push(convert_pcm_f32_to_16(*i));
            }
        },
        2 => {
            let l: Vec<f32> = samples.iter().step_by(2).map(|i| convert_pcm_16_to_f32(*i)).collect();
            let r: Vec<f32> = samples.iter().skip(1).step_by(2).map(|i| convert_pcm_16_to_f32(*i)).collect();
            let result = resampler.process(&[l,r], None).unwrap();
            let iterator = result[0].iter().zip(result[1].iter());
            new_samples.reserve(iterator.clone().count());
            for i in iterator {
                new_samples.extend_from_slice(&[convert_pcm_f32_to_16(*i.0), convert_pcm_f32_to_16(*i.1)]);
            }
        },
        _ => unreachable!()
    }

    new_samples
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
        return Err(ErrorMessage::AllocatedString(format!("Sound tag data \"{dir}\" directory contains mixed files and directories.", dir=data.to_string_lossy())));
    }
    if !contains_dirs && !contains_files {
        return Err(ErrorMessage::AllocatedString(format!("Sound data \"{dir}\" directory is empty.", dir=data.to_string_lossy())));
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
                return Err(ErrorMessage::AllocatedString(format!("Ambiguous pitch range \"{pitch_range}\" found (first directory is \"{pitch_range_1}\", second directory is \"{pitch_range_2}\")",
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
                    return Err(ErrorMessage::AllocatedString(format!("Ambiguous permutation \"{permutation}\" in \"{pitch_range}\" found (first file is \"{permutation_1}\", second file is \"{permutation_2}\")",
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
