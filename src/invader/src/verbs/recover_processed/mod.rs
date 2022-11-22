use ringhopper_proc::*;
use std::num::NonZeroUsize;
use std::process::ExitCode;
use crate::cmd::*;
use ringhopper::engines::h1::TagGroup;
use ringhopper::error::ErrorMessage;
use ringhopper::file::TagFile;
use crate::file::*;
use macros::terminal::*;
use ringhopper::error::ErrorMessageResult;
use std::path::*;

mod bitmap;
mod model;

pub enum RecoverProcessedResult {
    Recovered,
    DataAlreadyExists,
    SourceDataExists
}

const RECOVER_PROCESSED_FUNCTION_GROUPS: &'static [(TagGroup, fn (tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool, force: bool) -> ErrorMessageResult<RecoverProcessedResult>)] = &[
    (TagGroup::Bitmap, bitmap::recover_processed_bitmaps),
    (TagGroup::GBXModel, model::recover_processed_gbxmodels),
    (TagGroup::Model, model::recover_processed_models),
];

#[derive(Clone)]
struct RecoverProcessedOptions {
    batching: bool,
    force: bool,
    overwrite: bool,
    data_dir: PathBuf
}

fn recover_processed_tag(tag_file: &TagFile, log_mutex: super::LogMutex, _available_threads: NonZeroUsize, options: &RecoverProcessedOptions) -> ErrorMessageResult<bool> {
    let group = tag_file.tag_path.get_group();
    let file_data = read_file(&tag_file.file_path)?;

    for fg in RECOVER_PROCESSED_FUNCTION_GROUPS {
        if fg.0 == group {
            let result = fg.1(&file_data, tag_file, &options.data_dir, options.overwrite, options.force)?;
            let skipped;

            let l = log_mutex.lock();
            match result {
                RecoverProcessedResult::Recovered => {
                    println_success!(get_compiled_string!("engine.h1.verbs.recover.recovered_tag"), tag=tag_file.tag_path);
                    skipped = false;
                },
                RecoverProcessedResult::DataAlreadyExists => {
                    if !options.batching {
                        println!(get_compiled_string!("engine.h1.verbs.recover.skipped_tag_exists"), tag=tag_file.tag_path);
                    }
                    skipped = true;
                },
                RecoverProcessedResult::SourceDataExists => {
                    if !options.batching {
                        println_warn!(get_compiled_string!("engine.h1.verbs.recover-processed.skipped_tag_source_data"), tag=tag_file.tag_path);
                    }
                    skipped = true;
                },
            }
            drop(l);

            return Ok(skipped);
        }
    }

    // Can't recover this
    if !options.batching {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.recover.unable_to_recover_tag"), input_group=tag_file.tag_path.get_group())));
    }

    Ok(true)
}

pub fn recover_processed_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args,
                                                       &[Argument { long: "force", short: 'f', description: get_compiled_string!("engine.h1.verbs.recover-processed.arguments.force.description"), parameter: None, multiple: false }],
                                                       &[get_compiled_string!("arguments.specifier.tag_batch_with_group")],
                                                       executable,
                                                       verb.get_description(),
                                                       ArgumentConstraints::new().needs_tags()
                                                                                 .multiple_tags_directories()
                                                                                 .can_overwrite()
                                                                                 .needs_data())?;

    let tag_path = &parsed_args.extra[0];
    let options = RecoverProcessedOptions {
        force: parsed_args.named.contains_key("force"),
        batching: TagFile::uses_batching(tag_path),
        overwrite: parsed_args.named.get("overwrite").is_some(),
        data_dir: Path::new(&parsed_args.named["data"][0]).to_owned()
    };

    Ok(super::do_with_batching_threaded(recover_processed_tag, &tag_path, None, &str_slice_to_path_vec(&parsed_args.named["tags"]), parsed_args.threads, options)?.exit_code())
}
