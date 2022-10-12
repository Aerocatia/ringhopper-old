extern crate ringhopper;
use ringhopper::engines::*;
use ringhopper::cmd::*;

use std::str::FromStr;
use ringhopper::error::*;

extern crate strings;
extern crate macros;
extern crate flate2;
extern crate tiff;

use strings::*;

mod verbs;
use verbs::*;

mod file;
mod string;

use std::process::ExitCode;

/// [EngineModuleFn] interface for Halo: Combat Evolved.
#[derive(Default)]
pub struct HaloCE {}

impl EngineModuleFn for HaloCE {
    fn get_verb_function(&self, verb: Verb) -> Option<VerbFn> {
        match verb {
            Verb::Bitmap => Some(bitmap::bitmap_verb),
            Verb::Convert => Some(convert::convert_verb),
            Verb::ListEngines => Some(list_engines::list_engines_verb),
            Verb::Recover => Some(recover::recover_verb),
            Verb::Script => Some(script::script_verb),
            Verb::Strip => Some(strip::strip_verb),
            Verb::Strings => Some(unicode_strings::unicode_strings_verb),
            Verb::TagCollection => Some(collection::collection_verb),
            Verb::UICollection => Some(collection::collection_verb),
            Verb::UnicodeStrings => Some(unicode_strings::unicode_strings_verb),

            _ => None
        }
    }
    fn get_version(&self) -> &'static str {
        env!("invader_version")
    }
}

fn main() -> ExitCode {
    ringhopper::cmd::main_fn(&HaloCE::default())
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
