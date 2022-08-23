use engine::ExitCode;
use std::{path::Path, io::Read};

use crate::h1::unicode_string_list::{UnicodeStringList, UnicodeStringListString};

pub fn string_verb(args: &[&str]) -> ExitCode {
    let mut tags = Path::new("tags");
    let mut data = Path::new("data");

    // basic argument parser (todo: write a decent one)
    let mut args_vec = args.to_owned();
    'arg_loop: loop {
        for i in 0..args_vec.len() {
            if args_vec[i] == "--tags" || args_vec[i] == "-t" {
                tags = Path::new(args_vec[i + 1]);
                args_vec.remove(i + 1);
                args_vec.remove(i);
                continue 'arg_loop;
            }
            if args_vec[i] == "--data" || args_vec[i] == "-d" {
                data = Path::new(args_vec[i + 1]);
                args_vec.remove(i + 1);
                args_vec.remove(i);
                continue 'arg_loop;
            }
        }
        break;
    }

    if args_vec.len() != 1 {
        println!("Usage: string [-t <tags>] [-d <data>] <path>");
        return ExitCode::FAILURE;
    }

    let internal_path = args_vec[0].to_owned();
    let internal_path_path = Path::new(&internal_path).to_owned();

    if !tags.exists() {
        eprintln!("{} does not exist!", tags.to_str().unwrap());
        return ExitCode::FAILURE;
    }
    if !data.exists() {
        eprintln!("{} does not exist!", data.to_str().unwrap());
        return ExitCode::FAILURE;
    }

    let data_path = data.join(internal_path_path.clone()).with_extension("txt");
    let tag_path = tags.join(internal_path_path.clone()).with_extension("unicode_string_list");

    let mut data_file = match std::fs::File::open(data_path.to_owned()) {
        Ok(n) => n,
        Err(error) => {
            eprintln!("Error opening file: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut file_data = Vec::<u8>::new();
    match data_file.read_to_end(&mut file_data) {
        Ok(_) => (),
        Err(error) => {
            eprintln!("Error reading file: {error}");
            return ExitCode::FAILURE;
        }
    }
    drop(data_file);

    if file_data[0] == 0xFE || file_data[0] == 0xFF {
        eprintln!("UTF-16 input is not yet supported!");
        return ExitCode::FAILURE;
    }

    let string = match String::from_utf8(file_data) {
        Ok(n) => n,
        Err(error) => {
            eprintln!("Error parsing file: {error}");
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
        eprintln!("A '###END-STRING###' is missing!");
        return ExitCode::FAILURE;
    }

    match std::fs::create_dir_all(tags.parent().unwrap()) {
        Ok(_) => (),
        Err(error) => {
            eprintln!("Failed to create directories: {error}");
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

    let file = match crate::h1::ParsedTagFile::into_tag(&list, crate::h1::TagGroup::UnicodeStringList) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };

    let mut tag_file = match std::fs::File::create(tag_path.to_owned()) {
        Ok(n) => n,
        Err(error) => {
            eprintln!("Error opening final tag: {}", error);
            return ExitCode::FAILURE;
        }
    };

    use std::io::Write;

    match tag_file.write_all(&file) {
        Ok(_) => {
            println!("Wrote {}", tag_path.to_str().unwrap());
            ExitCode::SUCCESS
        },
        Err(e) => {
            println!("Couldn't write final tag data: {e}");
            ExitCode::FAILURE
        }
    }
}
