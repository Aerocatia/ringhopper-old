use std::num::{NonZeroU32, NonZeroUsize};
use std::process::ExitCode;
use std::path::*;
use std::sync::{Arc, Mutex};
use macros::println_success;
use ringhopper::engines::h1::definitions::{SoundClass, SoundFormat, Sound, SoundChannelCount, SoundSampleRate, SoundPitchRange, SoundPermutation};
use ringhopper::error::{ErrorMessageResult, ErrorMessage};
use ringhopper::file::*;
use ringhopper::types::*;
use ringhopper::engines::h1::*;
use ringhopper_proc::*;
use vorbis_rs::VorbisBitrateManagementStrategy;
use crate::file::*;
use crate::*;

mod util;

#[derive(Clone)]
struct SoundOptions {
    batched: bool,
    data_dir: PathBuf,
    sample_rate: Option<Option<u32>>,
    channel_count: Option<Option<usize>>,
    split: Option<bool>,
    adpcm_block_size: Option<bool>,
    compression_level: VorbisBitrateManagementStrategy,
    class: Option<SoundClass>,
    format: Option<SoundFormat>
}

pub fn sound_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args, &[
        Argument { long: "channel-count", short: 'C', description: get_compiled_string!("engine.h1.verbs.sound.arguments.channel-count.description"), parameter: Some("mono/stereo"), multiple: false },
        Argument { long: "class", short: 'c', description: get_compiled_string!("engine.h1.verbs.sound.arguments.class.description"), parameter: Some("class"), multiple: false },
        Argument { long: "compression-level", short: 'L', description: get_compiled_string!("engine.h1.verbs.sound.arguments.compression-level.description"), parameter: Some("level"), multiple: false },
        Argument { long: "format", short: 'f', description: get_compiled_string!("engine.h1.verbs.sound.arguments.format.description"), parameter: Some("format"), multiple: false },
        Argument { long: "sample-rate", short: 'R', description: get_compiled_string!("engine.h1.verbs.sound.arguments.sample-rate.description"), parameter: Some("Hz"), multiple: false },
        Argument { long: "split", short: 'S', description: get_compiled_string!("engine.h1.verbs.sound.arguments.split.description"), parameter: Some("on/off"), multiple: false },
        Argument { long: "fit-to-adpcm-block-size", short: 'A', description: get_compiled_string!("engine.h1.verbs.sound.arguments.fit-to-adpcm-block-size.description"), parameter: Some("on/off"), multiple: false },
    ], &[get_compiled_string!("arguments.specifier.tag_batch_without_group")], executable, verb.get_description(), ArgumentConstraints::new().needs_tags().needs_data().uses_threads())?;

    let tag_path = &parsed_args.extra[0];
    let options = SoundOptions {
        batched: TagFile::uses_batching(tag_path),
        data_dir: Path::new(&parsed_args.named["data"][0]).to_owned(),
        split: parsed_args.parse_bool_on_off("split")?,
        compression_level: match parsed_args.named.get("compression-level") {
            Some(n) => {
                let v = &n[0];
                if v.ends_with("k") {
                    VorbisBitrateManagementStrategy::Vbr {
                        target_bitrate: v[..v.len()-1].parse::<u32>()
                                                      .map_err(|_| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_bad_quality"), value=v)))
                                                      .map(|f| NonZeroU32::new(f.clamp(50,500) * 1000).unwrap())?
                    }
                }
                else {
                    VorbisBitrateManagementStrategy::QualityVbr {
                        target_quality: v.parse()
                                         .map_err(|_| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_bad_quality"), value=v)))
                                         .map(|f| if f < -0.2 || f > 1.0 {
                                                      Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_bad_quality_range"), value=v)))
                                                  }
                                                  else {
                                                      Ok(f)
                                                  }
                                         )??
                    }
                }
            },
            None => VorbisBitrateManagementStrategy::QualityVbr { target_quality: 0.8 }
        },
        class: parsed_args.parse_enum("class")?,
        format: parsed_args.parse_enum("format")?,
        channel_count: parsed_args.parse_set("channel-count", &[("stereo", Some(2)), ("mono", Some(1)), ("auto", None)])?,
        sample_rate: parsed_args.parse_set("sample-rate", &[("22050", Some(22050)), ("44100", Some(44100)), ("auto", None)])?,
        adpcm_block_size: parsed_args.parse_bool_on_off("fit-to-adpcm-block-size")?
    };

    let result = super::do_with_batching_threaded(do_single_sound, &tag_path, Some(TagGroup::Sound), &str_slice_to_path_vec(&parsed_args.named["tags"]), parsed_args.threads, options)?;
    Ok(result.exit_code())
}

fn do_single_sound(tag: &TagFile, log_mutex: super::LogMutex, available_threads: NonZeroUsize, options: &SoundOptions) -> ErrorMessageResult<bool> {
    let default_channel_count: Option<usize>;
    let default_sample_rate: Option<u32>;
    let available_threads = available_threads.get();

    // Load our sounds
    let mut sound_tag = if tag.file_path.is_file() {
        let sound_tag = *Sound::from_tag_file(&read_file(&tag.file_path)?)?.data;
        default_channel_count = options.channel_count.unwrap_or(match sound_tag.channel_count { SoundChannelCount::Mono => Some(1), SoundChannelCount::Stereo => Some(2) });
        default_sample_rate = options.sample_rate.unwrap_or(match sound_tag.sample_rate { SoundSampleRate::_22050Hz => Some(22050), SoundSampleRate::_44100Hz => Some(44100) });
        sound_tag
    }
    else {
        if options.class.is_none() {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_no_sound_class_given"), tag=tag.tag_path)))
        }
        default_channel_count = options.channel_count.unwrap_or(None);
        default_sample_rate = options.sample_rate.unwrap_or(None);
        Sound::default()
    };

    sound_tag.flags.split_long_sound_into_permutations = options.split.unwrap_or(sound_tag.flags.split_long_sound_into_permutations);
    sound_tag.flags.fit_to_adpcm_blocksize = options.adpcm_block_size.unwrap_or(sound_tag.flags.fit_to_adpcm_blocksize);
    sound_tag.format = options.format.unwrap_or(sound_tag.format);
    sound_tag.sound_class = options.class.unwrap_or(sound_tag.sound_class);

    let data = options.data_dir.join(tag.tag_path.get_path_without_extension());
    if !data.is_dir() {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_cannot_find_dir"), dir=data.to_string_lossy())))
    }

    let mut pitch_ranges = util::load_data_dir(&data)?;

    // Determine our sample/channel count
    let (highest_sample_rate, highest_channel_count) = util::get_best_sample_rate_and_channel_count(&pitch_ranges)?;
    let best_channel_count = default_channel_count.unwrap_or(highest_channel_count);
    let best_sample_rate = default_sample_rate.unwrap_or_else(|| match highest_sample_rate { n if n <= 22050 => 22050, _ => 44100 });
    let split = sound_tag.flags.split_long_sound_into_permutations;

    // Copy over tag data
    for pr in &mut pitch_ranges {
        for pr_tag in &mut sound_tag.pitch_ranges {
            if pr.name == pr_tag.name.to_str() {
                pr.pitch_bounds = pr_tag.bend_bounds;
                pr.natural_pitch = pr_tag.natural_pitch;
                for pe in &mut pr.permutations {
                    for pe_tag in &pr_tag.permutations {
                        if pe.name == pe_tag.name.to_str() {
                            pe.gain = pe_tag.gain;
                            pe.skip_fraction = pe_tag.skip_fraction;
                            break;
                        }
                    }
                }
                break;
            }
        }
    }

    let generates_mouth_data = match sound_tag.sound_class {
        SoundClass::UnitDialog | SoundClass::ScriptedDialogPlayer | SoundClass::ScriptedDialogOther | SoundClass::ScriptedDialogForceUnspatialized => true,
        _ => false
    };

    let pitch_range_count = pitch_ranges.len();

    // Have an array of indices indicating the next permutation to process.
    //
    // If size == permutation count for the pitch range, move to the next one.
    let permutation_count_per_pitch_range: Vec<usize> = pitch_ranges.iter().map(|v| v.permutations.len()).collect();
    let next_permutation_to_process = Arc::new(Mutex::new(vec![0usize; pitch_range_count]));

    let pitch_ranges = Arc::new(Mutex::new(pitch_ranges));
    let mut threads = Vec::with_capacity(available_threads);
    for _ in 0..available_threads {
        let nptp_array = next_permutation_to_process.clone();
        let pr_array = pitch_ranges.clone();
        let best_sample_rate = best_sample_rate.clone();
        let best_channel_count = best_channel_count.clone();
        let sound_tag_format = sound_tag.format.clone();
        let split = split.clone();
        let fit_to_adpcm_blocksize = sound_tag.flags.fit_to_adpcm_blocksize.clone();
        let compression_level = options.compression_level.clone();
        let count_array = permutation_count_per_pitch_range.clone();
        let available_threads = available_threads;

        threads.push(std::thread::spawn(move || -> ErrorMessageResult<()> {
            for pri in 0..pitch_range_count {
                let permutation_count = count_array[pri];
                loop {
                    // Get the next permutation we can process
                    let mut nptp = nptp_array.lock().unwrap();
                    let latest = &mut nptp[pri];
                    if *latest == permutation_count {
                        break;
                    }

                    // Increment it
                    let permutation_index = latest.to_owned();
                    *latest += 1;

                    // Drop it
                    drop(nptp);

                    // Get our permutation
                    let mut pra = pr_array.lock().unwrap();
                    let mut pe = std::mem::take(&mut pra[pri].permutations[permutation_index]);
                    drop(pra);

                    // Remix
                    if pe.channels != best_channel_count {
                        pe.samples = util::remix(&pe.samples, pe.channels, best_channel_count);
                        pe.channels = best_channel_count;
                    }

                    // Resample
                    if pe.sample_rate != best_sample_rate {
                        pe.samples = util::resample(&pe.samples, pe.channels, best_sample_rate as f64 / pe.sample_rate as f64)?;
                        pe.sample_rate = best_sample_rate;
                    }

                    // Fit to Xbox ADPCM block size
                    if sound_tag_format == SoundFormat::XboxAdpcm && fit_to_adpcm_blocksize {
                        let alignment = 64 * best_channel_count;
                        let disparity = pe.samples.len() % alignment;
                        if disparity != 0 {
                            // Round up to the next chunk size
                            let samples_to_add = alignment - disparity;
                            let end_length = pe.samples.len().min(8192 * (pe.sample_rate as usize / 22050usize) * pe.channels);
                            let new_end_length = end_length + samples_to_add;
                            let end_offset = pe.samples.len() - end_length;
                            let mut new_end_resampled = util::resample(&pe.samples[end_offset..], pe.channels, new_end_length as f64 / end_length as f64)?;
                            new_end_resampled.resize(new_end_length, *new_end_resampled.last().unwrap());
                            pe.samples.resize(end_offset, 0);
                            pe.samples.append(&mut new_end_resampled);
                        }
                    }

                    // Generate mouth data
                    if generates_mouth_data {
                        pe.generate_mouth_data()
                    }

                    // Encode
                    pe.encode(sound_tag_format, split, compression_level, available_threads)?;

                    // Move it back
                    let mut pra = pr_array.lock().unwrap();
                    pra[pri].permutations[permutation_index] = pe;
                    drop(pra);
                }
            }

            Ok(())
        }));
    }
    for t in threads {
        t.join().unwrap()?;
    }

    // Now let's write our final result
    let mut pitch_ranges = pitch_ranges.lock().unwrap();
    sound_tag.pitch_ranges.blocks = Vec::with_capacity(pitch_ranges.len());
    sound_tag.sample_rate = match best_sample_rate { 44100 => SoundSampleRate::_44100Hz, 22050 => SoundSampleRate::_22050Hz, _ => unreachable!() };
    sound_tag.channel_count = match best_channel_count { 1 => SoundChannelCount::Mono, 2 => SoundChannelCount::Stereo, _ => unreachable!() };

    for pr in &mut *pitch_ranges {
        let mut pitch_range = SoundPitchRange::default();
        pitch_range.natural_pitch = pr.natural_pitch;
        pitch_range.bend_bounds = pr.pitch_bounds;
        pitch_range.name = String32::from_str(&pr.name)?;

        let subpermutation_count = if split {
            let mut total = 0;
            for i in &pr.permutations {
                total += i.encoded_samples.len();
            }
            total
        }
        else {
            pr.permutations.len()
        };

        pitch_range.permutations.blocks.reserve(subpermutation_count);

        // Check some hard limits
        const PERMUTATION_LIMIT: usize = u16::MAX as usize;
        const SUBPERMUTATION_LIMIT: usize = PERMUTATION_LIMIT - 1;
        if pr.permutations.len() > PERMUTATION_LIMIT {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_permutation_limit_exceeded"),
                                                             pitch_range=pr.name,
                                                             count=pr.permutations.len(),
                                                             limit=PERMUTATION_LIMIT)));
        }
        if split && subpermutation_count > SUBPERMUTATION_LIMIT {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.sound.error_subpermutation_limit_exceeded"),
                                                             pitch_range=pr.name,
                                                             count=subpermutation_count,
                                                             limit=SUBPERMUTATION_LIMIT)));
        }

        pitch_range.actual_permutation_count = pr.permutations.len() as u16;

        let make_permutation = |pe: &util::Sound| -> ErrorMessageResult<SoundPermutation> {
            let mut permutation = SoundPermutation::default();
            permutation.name = String32::from_str(&pe.name)?;
            permutation.gain = pe.gain;
            permutation.skip_fraction = pe.skip_fraction;
            permutation.format = sound_tag.format;
            Ok(permutation)
        };

        if split {
            // First initialize our permutations.
            for i in 0..pr.permutations.len() {
                pitch_range.permutations.blocks.push(make_permutation(&pr.permutations[i])?);
            }

            // Next initialize our sub-permutations and write our samples.
            let mut index = 0;
            for pe in &mut pr.permutations {
                let subpermutation_count = pe.encoded_samples.len();

                if !pe.mouth_data.is_empty() {
                    return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.sound.error_cannot_split_mouth_data")));
                }

                let mut path = Vec::with_capacity(subpermutation_count);
                path.push(index);
                index += 1;
                for _ in 1..subpermutation_count {
                    path.push(pitch_range.permutations.len());
                    pitch_range.permutations.blocks.push(make_permutation(&pe)?);
                }

                let mut i = 0;
                for s in &mut pe.encoded_samples {
                    let is_last = i + 1 == subpermutation_count;
                    let permutation = &mut pitch_range.permutations[path[i]];
                    i += 1;
                    permutation.samples.append(&mut s.0);
                    permutation.buffer_size = s.1 as u32;
                    if !is_last {
                        permutation.next_permutation_index = Some(path[i] as u16);
                    }
                }
            }
        }
        else {
            // Basically do a simple copy
            for pe in &mut pr.permutations {
                let mut permutation = make_permutation(&pe)?;
                let (samples, buffer_size) = &mut pe.encoded_samples[0];
                permutation.samples.append(samples);
                permutation.buffer_size = *buffer_size as u32;
                permutation.mouth_data.append(&mut pe.mouth_data);
                pitch_range.permutations.blocks.push(permutation);
            }
        }

        sound_tag.pitch_ranges.blocks.push(pitch_range);
    }

    // Show our output info
    if options.batched {
        let channel_count_to_str = |c| match c {
            1 => "mono",
            2 => "stereo",
            _ => unreachable!()
        };

        let mut total_time = 0.0;
        let mut total_size = 0usize;

        for pi in 0..pitch_ranges.len() {
            let pitch_range = &pitch_ranges[pi];
            let pitch_range_tag = &sound_tag.pitch_ranges[pi];
            println!(get_compiled_string!("engine.h1.verbs.sound.output_pitch_range_header"),
                     pitch_range_index=pi,
                     pitch_range_name=pitch_range.name,
                     permutation_count=pitch_range.permutations.len());

            for pm in 0..pitch_range.permutations.len() {
                let permutation = &pitch_range.permutations[pm];
                let total_samples = permutation.samples.len() / permutation.channels;
                let total_length_in_seconds = (total_samples as f32) / (permutation.sample_rate as f32);

                let min = (total_length_in_seconds / 60.0) as u32;
                let sec = (total_length_in_seconds % 60.0) as u32;
                let msec = ((total_length_in_seconds % 1.0) * 1000.0) as u32;

                total_time += total_length_in_seconds as f64;

                for permutation_tag in &pitch_range_tag.permutations {
                    if permutation_tag.name.to_str() == permutation.name {
                        total_size = total_size.saturating_add(permutation_tag.samples.len());
                    }
                }

                println!(get_compiled_string!("engine.h1.verbs.sound.output_pitch_range_permutation"),
                         permutation_index=pm,
                         permutation_name=permutation.name,
                         min=min,
                         sec=sec,
                         msec=msec,
                         original_sample_rate=permutation.original_sample_rate,
                         original_bits_per_sample=permutation.original_bits_per_sample,
                         original_codec=permutation.original_codec,
                         original_channel_count=channel_count_to_str(permutation.channels));
            }

            println!();
        }

        println!(get_compiled_string!("engine.h1.verbs.sound.output_end"),
                 sample_rate=best_sample_rate,
                 channel_count=channel_count_to_str(best_channel_count),
                 format=sound_tag.format.as_str_pretty(),
                 size=format_size(total_size),
                 kbps=(total_size as f64 / (1000.0 / 8.0)) / total_time,
                 split=match split { false => "", true => " (split)" })
    }

    make_parent_directories(&tag.file_path)?;
    write_file(&tag.file_path, &sound_tag.into_tag_file()?)?;

    let l = log_mutex.lock().unwrap();
    println_success!(get_compiled_string!("engine.h1.verbs.unicode-strings.saved_file"), file=tag.tag_path);
    drop(l);

    Ok(true)
}

