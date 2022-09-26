use strings::*;
use std::process::ExitCode;
use ringhopper::cmd::*;
use ringhopper::error::{ErrorMessageResult, ErrorMessage};
use ringhopper::file::*;
use crate::file::*;
use macros::terminal::*;
use ringhopper::engines::h1::definitions::parse_tag_file;

fn strip_tag(path: &std::path::Path) -> ErrorMessageResult<bool> {
    let file_data = read_file(path)?;
    let final_data = parse_tag_file(&file_data)?.data.into_tag_file()?;

    // No need to eat your drive
    if file_data == final_data {
        Ok(false)
    }
    else {
        // Write it
        write_file(path, &final_data)?;
        Ok(true)
    }
}

pub fn strip_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args, &[], &[get_compiled_string!("arguments.specifier.tag_batch_with_group")], executable, verb.get_description(), ArgumentConstraints::new().needs_tags().multiple_tags_directories())?;

    let tags = match TagFile::from_tag_path_batched(&str_slice_to_path_vec(&parsed_args.named["tags"]), &parsed_args.extra[0], None) {
        Ok(n) => n,
        Err(e) => panic!("{}", e)
    };

    let mut count = 0usize;
    let mut processed = 0usize;
    let mut error_count = 0usize;
    let total = tags.len();
    for i in tags {
        match strip_tag(&i.file_path) {
            Ok(written) => {
                if written {
                    println_success!(get_compiled_string!("engine.h1.verbs.strip.stripped_tag"), tag=i.tag_path);
                    processed += 1;
                }
                else {
                    println!(get_compiled_string!("engine.h1.verbs.strip.skipped_tag"), tag=i.tag_path);
                }
                count += 1;
            },
            Err(e) => {
                eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.strip.error_could_not_strip_tag"), tag=i.tag_path, error=e);
                error_count += 1;
            }
        }
    }

    if total == 0 {
        Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_no_tags_found"))))
    }
    else if count == 0 {
        Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.strip.error_no_tags_stripped"), error=error_count)))
    }
    else if error_count > 0 {
        println_warn!(get_compiled_string!("engine.h1.verbs.strip.stripped_some_tags_with_errors"), count=processed, error=error_count);
        Ok(ExitCode::FAILURE)
    }
    else {
        println_success!(get_compiled_string!("engine.h1.verbs.strip.stripped_all_tags"), count=processed);
        Ok(ExitCode::SUCCESS)
    }
}
