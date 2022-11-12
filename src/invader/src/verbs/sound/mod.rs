use std::num::NonZeroU32;
use std::process::ExitCode;
use std::path::*;
use macros::println_success;
use ringhopper::engines::h1::definitions::{SoundClass, SoundFormat, Sound, SoundChannelCount, SoundSampleRate, SoundPitchRange, SoundPermutation};
use ringhopper::error::{ErrorMessageResult, ErrorMessage};
use ringhopper::file::*;
use ringhopper::types::*;
use ringhopper::engines::h1::*;
use ringhopper::engines::h1::TagReference;
use ringhopper_proc::*;
use vorbis_rs::VorbisBitrateManagementStrategy;
use crate::file::*;
use crate::*;

mod util;

struct SoundOptions {
    sample_rate: Option<u32>,
    channel_count: Option<usize>,
    split: Option<bool>,
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
    ], &[get_compiled_string!("arguments.specifier.tag_without_group")], executable, verb.get_description(), ArgumentConstraints::new().needs_tags().needs_data().uses_threads())?;

    let options = SoundOptions {
        split: parsed_args.parse_bool_on_off("split")?,
        compression_level: match parsed_args.named.get("compression-level") {
            Some(n) => {
                let v = &n[0];
                if v.ends_with("k") {
                    VorbisBitrateManagementStrategy::Vbr {
                        target_bitrate: v[..v.len()-1].parse::<u32>()
                                                      .map_err(|_| ErrorMessage::AllocatedString(format!("Invalid compression quality \"{value}\": could not parse", value=v)))
                                                      .map(|f| NonZeroU32::new(f.clamp(50,500) * 1000).unwrap())?
                    }
                }
                else {
                    VorbisBitrateManagementStrategy::QualityVbr {
                        target_quality: v.parse()
                                         .map_err(|_| ErrorMessage::AllocatedString(format!("Invalid compression quality \"{value}\": could not parse", value=v)))
                                         .map(|f| if f < -0.2 || f > 1.0 {
                                                      Err(ErrorMessage::AllocatedString(format!("Invalid compression quality \"{value}\": not between -0.2 and 1.0", value=v)))
                                                  }
                                                  else {
                                                      Ok(f)
                                                  }
                                         )??
                    }
                }
            },
            None => VorbisBitrateManagementStrategy::QualityVbr { target_quality: 1.0 }
        },
        class: parsed_args.parse_enum("class")?,
        format: parsed_args.parse_enum("format")?,
        channel_count: parsed_args.parse_set("channel-count", &[("stereo", 2), ("mono", 1)])?,
        sample_rate: parsed_args.parse_set("sample-rate", &[("22050", 22050), ("44100", 44100)])?
    };

    let tag_path = &parsed_args.extra[0];
    let tag_path = TagReference::from_path_and_group(tag_path, TagGroup::Sound)?;
    let file_path = Path::new(&parsed_args.named["tags"][0]).join(tag_path.to_string());
    do_single_sound(&TagFile { tag_path, file_path }, Path::new(&parsed_args.named["data"][0]), &options, true)?;

    Ok(ExitCode::FAILURE)
}

fn do_single_sound(tag: &TagFile, data_dir: &Path, options: &SoundOptions, show_extended_info: bool) -> ErrorMessageResult<()> {
    let default_channel_count;
    let default_sample_rate;

    let mut sound_tag = if tag.file_path.is_file() {
        let sound_tag = *Sound::from_tag_file(&read_file(&tag.file_path)?)?.data;
        default_channel_count = Some(options.channel_count.unwrap_or(match sound_tag.channel_count { SoundChannelCount::Mono => 1, SoundChannelCount::Stereo => 2 }));
        default_sample_rate = Some(options.sample_rate.unwrap_or(match sound_tag.sample_rate { SoundSampleRate::_22050Hz => 22050, SoundSampleRate::_44100Hz => 44100 }));
        sound_tag
    }
    else {
        if options.class.is_none() {
            return Err(ErrorMessage::AllocatedString(format!("A sound class is required because sound tag \"{tag}\" does not yet exist.", tag=tag.tag_path)))
        }
        default_channel_count = options.channel_count;
        default_sample_rate = options.sample_rate;
        Sound::default()
    };

    sound_tag.flags.split_long_sound_into_permutations = options.split.unwrap_or(sound_tag.flags.split_long_sound_into_permutations);
    sound_tag.format = options.format.unwrap_or(sound_tag.format);
    sound_tag.sound_class = options.class.unwrap_or(sound_tag.sound_class);

    let data = data_dir.join(tag.tag_path.get_path_without_extension());
    if !data.is_dir() {
        return Err(ErrorMessage::AllocatedString(format!("Failed to find the sound tag's data directory. \"{dir}\" does not exist or is not a directory", dir=data.to_string_lossy())))
    }

    let mut pitch_ranges = util::load_data_dir(&data)?;

    // Determine our sample/channel count
    let (highest_sample_rate, highest_channel_count) = util::get_best_sample_rate_and_channel_count(&pitch_ranges)?;
    let best_channel_count = default_channel_count.unwrap_or(highest_channel_count);
    let best_sample_rate = default_sample_rate.unwrap_or_else(|| match highest_sample_rate { n if n <= 22050 => 22050, _ => 44100 });
    let split = sound_tag.flags.split_long_sound_into_permutations;

    // Resample / remix. Copy over tag data. Write mouth data if needed
    for pr in &mut pitch_ranges {
        for pe in &mut pr.permutations {
            // Remix
            if pe.channels != best_channel_count {
                pe.samples = util::remix(&pe.samples, pe.channels, best_channel_count);
                pe.channels = best_channel_count;
            }

            // Resample
            if pe.sample_rate != best_sample_rate {
                pe.samples = util::resample(&pe.samples, pe.channels, pe.sample_rate, best_sample_rate)?;
                pe.sample_rate = best_sample_rate;
            }
        }

        // Copy over tag data
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

        // Generate mouth data
        match sound_tag.sound_class {
            SoundClass::UnitDialog | SoundClass::ScriptedDialogPlayer | SoundClass::ScriptedDialogOther | SoundClass::ScriptedDialogForceUnspatialized => {
                for pe in &mut pr.permutations {
                    pe.generate_mouth_data()
                }
            },
            _ => ()
        }
    }

    // Encode
    for pr in &mut pitch_ranges {
        for pe in &mut pr.permutations {
            pe.encode(sound_tag.format, split, options.compression_level)?;
        }
    }

    // Now let's write our final result
    sound_tag.pitch_ranges.blocks = Vec::with_capacity(pitch_ranges.len());
    sound_tag.sample_rate = match best_sample_rate { 44100 => SoundSampleRate::_44100Hz, 22050 => SoundSampleRate::_22050Hz, _ => unreachable!() };
    sound_tag.channel_count = match best_channel_count { 1 => SoundChannelCount::Mono, 2 => SoundChannelCount::Stereo, _ => unreachable!() };

    for pr in &mut pitch_ranges {
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
            return Err(ErrorMessage::AllocatedString(format!("Pitch range \"{pitch_range}\" exceeds the maximum number of permutations allowed ({count} > {limit})",
                                                             pitch_range=pr.name,
                                                             count=pr.permutations.len(),
                                                             limit=PERMUTATION_LIMIT)));
        }
        if split && subpermutation_count > SUBPERMUTATION_LIMIT {
            return Err(ErrorMessage::AllocatedString(format!("Pitch range \"{pitch_range}\" exceeds the maximum number of sub-permutations allowed ({count} > {limit})",
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
                    return Err(ErrorMessage::StaticString("Cannot split permutations with generated mouth data."));
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
    if show_extended_info {
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
            println!("Pitch range #{pitch_range_index} ({pitch_range_name}): {permutation_count} permutation(s)",
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

                println!("    Permutation #{permutation_index} ({permutation_name}): {min:02}:{sec:02}.{msec:03} (input: {original_sample_rate} Hz, {original_channel_count}, {original_bits_per_sample}, {original_codec})",
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

        println!("Output: {sample_rate} Hz, {channel_count}, {format} @ {kbps:.01} kbps ({size})",
                 sample_rate=best_sample_rate,
                 channel_count=channel_count_to_str(best_channel_count),
                 format=sound_tag.format.as_str_pretty(),
                 size=format_size(total_size),
                 kbps=(total_size as f64 / (1000.0 / 8.0)) / total_time)
    }

    make_parent_directories(&tag.file_path)?;
    write_file(&tag.file_path, &sound_tag.into_tag_file()?)?;

    println_success!(get_compiled_string!("engine.h1.verbs.unicode-strings.saved_file"), file=tag.tag_path);

    Ok(())
}

