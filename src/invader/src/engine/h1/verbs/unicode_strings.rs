use engine::ExitCode;
use std::{path::Path, io::Read};
use crate::h1::unicode_string_list::{UnicodeStringList, UnicodeStringListString};
use crate::h1::types::{ParsedTagFile, TagGroup};
use crate::cmd::args::*;
use crate::terminal::*;
use strings::get_compiled_string;

pub fn unicode_strings_verb(verb: &crate::cmd::Verb, args: &[&str], executable: &str) -> ExitCode {
    let parsed_args = try_parse_arguments!(args, &[], &[get_compiled_string!("arguments.specifier.tag_without_group")], executable, verb.get_description(), ArgumentConstraints::new().needs_data().needs_tags());

    let tags = Path::new(&parsed_args.named.get("tags").unwrap()[0]);
    let data = Path::new(&parsed_args.named.get("data").unwrap()[0]);
    let internal_path = &parsed_args.extra[0];
    let internal_path_path = Path::new(&internal_path).to_owned();
    let data_path = data.join(internal_path_path.clone()).with_extension("txt");
    let tag_path = tags.join(internal_path_path.clone()).with_extension("unicode_string_list");

    let mut data_file = match std::fs::File::open(data_path.to_owned()) {
        Ok(n) => n,
        Err(error) => {
            eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_opening_file_read"), error=error, file=data_path.display());
            return ExitCode::FAILURE;
        }
    };

    let mut file_data = Vec::<u8>::new();
    match data_file.read_to_end(&mut file_data) {
        Ok(_) => (),
        Err(error) => {
            eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_reading_file"), error=error, file=data_path.display());
            return ExitCode::FAILURE;
        }
    }
    drop(data_file);

    if file_data[0] == 0xFE || file_data[0] == 0xFF {
        eprintln_error_pre!("UTF-16 input is not yet supported!");
        return ExitCode::FAILURE;
    }

    let string = match String::from_utf8(file_data) {
        Ok(n) => n,
        Err(error) => {
            eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_parsing_file"), error=error, file=data_path.display());
            return ExitCode::FAILURE;
        }
    };

    let mut current_string = String::new();
    let mut strings = Vec::<String>::new();
    for mut l in string.lines() {
        if l.ends_with("\n") {
            l = &l[0..l.len()-1];
        }
        if l.ends_with("\r") {
            l = &l[0..l.len()-1];
        }
        if l == "###END-STRING###" {
            strings.push(current_string);
            current_string = String::new();
            continue;
        }
        if !current_string.is_empty() {
            current_string += "\r\n";
        }
        current_string += l;
    }
    if !current_string.is_empty() {
        eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_missing_end_string"));
        return ExitCode::FAILURE;
    }

    match std::fs::create_dir_all(tags.parent().unwrap()) {
        Ok(_) => (),
        Err(error) => {
            eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_creating_directories"), error=error, dirs=tags.parent().unwrap().display());
            return ExitCode::FAILURE;
        }
    }

    let mut list = UnicodeStringList::default();
    for s in strings {
        let mut v: Vec<u16> = s.encode_utf16().collect();
        v.push(0); // null terminator

        let mut v_8 = Vec::<u8>::new();
        for b in v {
            v_8.push((b & 0xFF) as u8);
            v_8.push(((b >> 8) & 0xFF) as u8);
        }
        list.strings.blocks.push(UnicodeStringListString { string_data: v_8 })
    }

    let file = match ParsedTagFile::into_tag(&list, TagGroup::UnicodeStringList) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };

    let mut tag_file = match std::fs::File::create(tag_path.to_owned()) {
        Ok(n) => n,
        Err(error) => {
            eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_opening_file_write"), error=error, file=tag_path.display());
            return ExitCode::FAILURE;
        }
    };

    use std::io::Write;

    match tag_file.write_all(&file) {
        Ok(_) => {
            println_success!(get_compiled_string!("engine.h1.verbs.unicode-strings.saved_file"), file=tag_path.display());
            ExitCode::SUCCESS
        },
        Err(e) => {
            eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_writing_file"), error=e, file=tag_path.display());
            ExitCode::FAILURE
        }
    }
}
