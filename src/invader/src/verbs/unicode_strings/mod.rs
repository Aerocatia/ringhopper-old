use std::process::ExitCode;
use std::path::Path;
use ringhopper::engines::h1::definitions::{UnicodeStringList, UnicodeStringListString};
use ringhopper::engines::h1::*;
use crate::cmd::*;
use ringhopper::types::tag::TagGroupFn;
use macros::terminal::*;
use crate::file::*;
use ringhopper::error::{ErrorMessage, ErrorMessageResult};
use strings::get_compiled_string;

extern crate encoding;
use self::encoding::{Encoding, EncoderTrap};
use self::encoding::all::WINDOWS_1252;
use std::ffi::CString;

fn make_string_list(file_data: &[u8], data_path: &Path, group: TagGroup) -> ErrorMessageResult<UnicodeStringList> {
    // If we're making regular 8-bit strings, parse as 1252
    let string = if group == TagGroup::StringList {
        match WINDOWS_1252.decode(&file_data, encoding::DecoderTrap::Strict) {
            Ok(n) => n,
            Err(error) => {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_parsing_file"), error=error, file=data_path.display())));
            }
        }
    }

    // Check if UTF-16. If so, parse it as such.
    else if file_data.len() < 2 {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_parsing_file"), error="cannot determine encoding of input", file=data_path.display())));
    }
    else if (file_data[0] == 0xFE && file_data[1] == 0xFF) || (file_data[0] == 0xFF && file_data[1] == 0xFE) {
        let mut string_data_as_16 = Vec::<u16>::new();

        if file_data.len() % 2 != 0 {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_parsing_file"), error="invalid UTF-16 input", file=data_path.display())));
        }

        // Get which function we need to use to read the bytes
        let read_fn = match file_data[0] {
            0xFE => u16::from_be_bytes,
            0xFF => u16::from_le_bytes,
            _ => unreachable!()
        };

        // Go two bytes at a time
        for s in (2..file_data.len()).step_by(2) {
            use std::convert::TryInto;

            let bytes: [u8; 2] = file_data[s..s+2].try_into().unwrap();
            let data  = read_fn(bytes);
            string_data_as_16.push(data);
        }

        // Read the UTF-16 data.
        match String::from_utf16(&string_data_as_16) {
            Ok(n) => n,
            Err(e) => {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_parsing_file"), error=e, file=data_path.display())));
            }
        }
    }

    // Otherwise, parse as UTF8.
    else {
        match std::str::from_utf8(&file_data) {
            Ok(n) => n.to_owned(),
            Err(error) => {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_parsing_file"), error=error, file=data_path.display())));
            }
        }
    };

    // Go through it line-by-line and put together the string data as UTF-16 with CRLF line endings (except for the last line)
    //
    // TODO: Use intersperse when [std::iter::Intersperse] is stable (see https://github.com/rust-lang/rust/issues/79524)
    let mut current_string = None;
    let mut strings = Vec::<String>::new();
    for l in string.lines() {
        if l == "###END-STRING###" {
            strings.push(current_string.unwrap_or_default());
            current_string = None;
            continue;
        }
        if current_string.is_some() {
            current_string = Some(current_string.unwrap() + "\r\n" + l);
        }
        else {
            current_string = Some(l.to_owned())
        }
    }

    // If we started a string without an ###END-STRING### to close it, this is an error.
    if current_string.is_some() {
        return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.unicode-strings.error_missing_end_string")));
    }

    let mut list = UnicodeStringList::default();
    match group {
        TagGroup::StringList => {
            for s in strings {
                list.strings.blocks.push(UnicodeStringListString {
                    string: CString::new(
                        match WINDOWS_1252.encode(&s, EncoderTrap::Strict) {
                            Ok(n) => n,
                            Err(e) => {
                                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_string_unable_to_encode_into_1252"), error=e)));
                            }
                        }
                    ).unwrap().into_bytes_with_nul()
                })
            }
        },
        TagGroup::UnicodeStringList => {
            for s in strings {
                let mut v: Vec<u16> = s.encode_utf16().collect();
                v.push(0); // null terminator

                let mut v_8 = Vec::<u8>::new();
                for b in v {
                    v_8.push((b & 0xFF) as u8);
                    v_8.push(((b >> 8) & 0xFF) as u8);
                }
                list.strings.blocks.push(UnicodeStringListString { string: v_8 })
            }
        },
        _ => unreachable!()
    };

    Ok(list)
}

pub fn unicode_strings_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args, &[], &[get_compiled_string!("arguments.specifier.tag_without_group")], executable, verb.get_description(), ArgumentConstraints::new().needs_data().needs_tags())?;

    let tags = Path::new(&parsed_args.named["tags"][0]);
    let data = Path::new(&parsed_args.named["data"][0]);
    let internal_path = &parsed_args.extra[0];
    let internal_path_path = Path::new(&internal_path).to_owned();
    let data_path = data.join(internal_path_path.clone()).with_extension("txt");
    let file_data = read_file(&data_path)?;
    let group = match *verb { Verb::Strings => TagGroup::StringList, Verb::UnicodeStrings => TagGroup::UnicodeStringList, _ => unreachable!() };
    let list = make_string_list(&file_data, &data_path, group)?;

    let output_tag = ParsedTagFile::into_tag(&list, group)?;
    let tag_path = tags.join(internal_path_path.clone()).with_extension(group.as_str());
    make_parent_directories(&tag_path)?;
    write_file(&tag_path, &output_tag)?;

    println_success!(get_compiled_string!("engine.h1.verbs.unicode-strings.saved_file"), file=tag_path.display());
    Ok(ExitCode::SUCCESS)
}
