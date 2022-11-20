use macros::println_success;
use ringhopper_proc::*;
use std::num::NonZeroUsize;
use std::process::ExitCode;
use crate::cmd::*;
use macros::terminal::*;
use ringhopper::error::ErrorMessageResult;
use ringhopper::file::*;
use crate::file::*;
use ringhopper::engines::h1::definitions::parse_tag_file;

fn strip_tag(path: &TagFile, log_mutex: super::LogMutex, _: NonZeroUsize, batched: &bool) -> ErrorMessageResult<bool> {
    let file_data = read_file(&path.file_path)?;
    let final_data = parse_tag_file(&file_data)?.data.into_tag_file()?;
    let skip = file_data == final_data;

    if !skip {
        write_file(&path.file_path, &final_data)?;
    }

    let l = log_mutex.lock();
    if !skip {
        println_success!(get_compiled_string!("engine.h1.verbs.strip.stripped_tag"), tag=path.tag_path);
    }
    if skip && !*batched {
        println!(get_compiled_string!("engine.h1.verbs.strip.skipped_tag"), tag=path.tag_path);
    }
    drop(l);

    Ok(!skip)
}

pub fn strip_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args, &[], &[get_compiled_string!("arguments.specifier.tag_batch_with_group")], executable, verb.get_description(), ArgumentConstraints::new().needs_tags().uses_threads().multiple_tags_directories())?;
    let tag_path = &parsed_args.extra[0];
    Ok(super::do_with_batching_threaded(strip_tag, tag_path, None, &str_slice_to_path_vec(&parsed_args.named["tags"]), parsed_args.threads, TagFile::uses_batching(tag_path))?.exit_code())
}
