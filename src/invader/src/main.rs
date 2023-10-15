extern crate ringhopper;

use std::str::FromStr;
use ringhopper::error::*;

extern crate ringhopper_proc;
extern crate macros;
extern crate flate2;
extern crate tiff;
extern crate png;
extern crate symphonia;
extern crate vorbis_rs;
extern crate xbadpcm;
extern crate libsamplerate_sys;
extern crate jxl_oxide;

use ringhopper_proc::*;

mod cmd;
use cmd::*;

use ringhopper::error::ErrorMessageResult;
use std::process::ExitCode;
use macros::terminal::*;

mod verbs;
use verbs::*;

mod file;
mod string;

fn print_usage(path: &str, lookup: &str) {
    eprintln!("{}", env!("invader_version"));

    eprintln!(get_compiled_string!("command_usage.error"), path=path);
    eprintln!();

    if !lookup.is_empty() {
        eprintln_error_pre!(get_compiled_string!("command_usage.error_no_verbs_matched"), lookup=lookup)
    }

    eprintln!(get_compiled_string!("command_usage.error_available_verbs"));

    let mut verbs_listed = 0usize;
    for v in ALL_VERBS {
        if get_verb_function(v.verb).is_some() {
            verbs_listed += 1;
            eprint!("    {: <20}  ", v.verb.get_name());
            let pos = 4 + 15 + 2 + 3 + 2;
            print_word_wrap(v.verb.get_description(), pos, pos, OutputType::Stderr);
            eprintln!();
        }
    }

    if verbs_listed == 0 {
        eprintln_warn!("    {}", get_compiled_string!("command_usage.error_no_verbs_available"));
    }

    eprintln!();
    eprintln!(get_compiled_string!("command_usage.error_get_help"), path=path);
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let mut args_ref: Vec<&str> = Vec::new();
    for a in &args {
        args_ref.push(a);
    }

    // No arguments?
    if args.len() == 1 {
        print_usage(args_ref[0], "");
        ExitCode::from(1)
    }

    // Try to match an argument then!
    else if let Some(v) = Verb::from_input(args_ref[1]) {
        if let Some(f) = get_verb_function(v) {
            match f(&v, &args_ref[2..], &format!("{} {}", args_ref[0], v.get_name())) {
                Ok(n) => n,
                Err(e) => {
                    let string = e.to_string();
                    if !string.is_empty() {
                        eprintln_error_pre!("{string}");
                    }

                    ExitCode::FAILURE
                }
            }
        }
        else {
            eprintln_error_pre!(get_compiled_string!("command_usage.error_verb_unsupported"), verb=v.get_name());
            ExitCode::from(2)
        }
    }
    else {
        print_usage(&args[0], &args[1]);
        ExitCode::from(2)
    }
}

fn get_verb_function(verb: Verb) -> Option<VerbFn> {
    match verb {
        Verb::Bitmap => Some(bitmap::bitmap_verb),
        Verb::Convert => Some(convert::convert_verb),
        Verb::Lightmap => Some(lightmap::lightmap_verb),
        Verb::ListEngines => Some(list_engines::list_engines_verb),
        Verb::NormalizeLightmaps => Some(normalize_lightmaps::normalize_lightmaps_verb),
        Verb::Recover => Some(recover::recover_verb),
        Verb::RecoverProcessed => Some(recover_processed::recover_processed_verb),
        Verb::Sound => Some(sound::sound_verb),
        Verb::Script => Some(script::script_verb),
        Verb::Strip => Some(strip::strip_verb),
        Verb::Strings => Some(unicode_strings::unicode_strings_verb),
        Verb::TagCollection => Some(collection::collection_verb),
        Verb::UICollection => Some(collection::collection_verb),
        Verb::UnicodeStrings => Some(unicode_strings::unicode_strings_verb),
        Verb::UpscaleHUD => Some(upscale_hud::upscale_hud_verb),

        _ => None
    }
}

/// Parse a string into a value.
///
/// This calls `T::from_str(s)` and can be used on anything where [`FromStr`]'s error type is [`ErrorMessage`] such as tag enums.
fn from_str<T: FromStr<Err = ErrorMessage>>(s: &str) -> ErrorMessageResult<T> {
    T::from_str(s)
}

/// Load a bitmap color plate from compressed color plate data.
fn load_bitmap_color_plate(tag: &ringhopper::engines::h1::definitions::Bitmap) -> ErrorMessageResult<Vec<u8>> {
    use std::convert::TryInto;
    use flate2::{Decompress, Status, FlushDecompress};

    // Do we even have this color plate?
    if tag.compressed_color_plate_data.len() < 4 {
        return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover.error_bitmap_color_plate_data_invalid")));
    }

    let width = tag.color_plate_width as usize;
    let height = tag.color_plate_height as usize;
    let length = u32::from_be_bytes(tag.compressed_color_plate_data[0..4].try_into().unwrap()) as usize;

    // Check if width * height * 4 is not equal to length, also accounting for integer overflowing (in which case it would also not be equal).
    if (|| if width.checked_mul(height)?.checked_mul(4)? != length { None } else { Some(()) } )().is_none() {
        return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover.error_bitmap_color_plate_data_invalid")));
    }

    // Try to decompress with deflate
    let mut decompressed_output = Vec::<u8>::new();
    decompressed_output.resize(length, 0);
    let mut decompressor = Decompress::new(true);
    match decompressor.decompress(&tag.compressed_color_plate_data[4..], &mut decompressed_output, FlushDecompress::None).unwrap() {
        Status::StreamEnd => (),
        _ => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover.error_bitmap_color_plate_data_invalid")))
    }

    Ok(decompressed_output)
}

fn make_tiff(pixels_r8g8b8a8: &[u8], width: usize, height: usize) -> Vec<u8> {
    use tiff::encoder::*;
    use std::io::Cursor;

    // Encode into a TIFF
    let mut data = Vec::new();
    let mut encoder = TiffEncoder::new(Cursor::new(&mut data)).unwrap();
    let mut image = encoder.new_image::<colortype::RGBA8>(width as u32, height as u32).unwrap();
    image.encoder().write_tag(tiff::tags::Tag::ExtraSamples, &[2u16][..]).unwrap();
    image.rows_per_strip(2).unwrap();

    // Write each strip
    let mut idx = 0;
    while image.next_strip_sample_count() > 0 {
        let sample_count = image.next_strip_sample_count() as usize;
        image.write_strip(&pixels_r8g8b8a8[idx..idx+sample_count]).unwrap();
        idx += sample_count;
    }

    // Done
    image.finish().unwrap();

    data
}
