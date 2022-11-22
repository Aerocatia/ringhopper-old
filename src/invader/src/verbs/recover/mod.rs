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
mod hud_message_text;
mod model;
mod scenario;
mod string_list;
mod tag_collection;

pub enum RecoverResult {
    Recovered,
    DataAlreadyExists,
    NoSourceData
}

const RECOVER_FUNCTION_GROUPS: &'static [(TagGroup, fn (tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult>)] = &[
    (TagGroup::Bitmap, bitmap::recover_bitmaps),
    (TagGroup::GBXModel, model::recover_gbxmodels),
    (TagGroup::HUDMessageText, hud_message_text::recover_hud_messages),
    (TagGroup::Model, model::recover_models),
    (TagGroup::Scenario, scenario::recover_scripts),
    (TagGroup::StringList, string_list::recover_string_list),
    (TagGroup::TagCollection, tag_collection::recover_tag_collection),
    (TagGroup::UIWidgetCollection, tag_collection::recover_ui_widget_collection),
    (TagGroup::UnicodeStringList, string_list::recover_unicode_string_list)
];

#[derive(Clone)]
struct RecoverOptions {
    batching: bool,
    overwrite: bool,
    data_dir: PathBuf
}

fn recover_tag(tag_file: &TagFile, log_mutex: super::LogMutex, _available_threads: NonZeroUsize, options: &RecoverOptions) -> ErrorMessageResult<bool> {
    let group = tag_file.tag_path.get_group();
    let file_data = read_file(&tag_file.file_path)?;

    for fg in RECOVER_FUNCTION_GROUPS {
        if fg.0 == group {
            let result = fg.1(&file_data, tag_file, &options.data_dir, options.overwrite)?;
            let skipped;

            let l = log_mutex.lock();
            match result {
                RecoverResult::Recovered => {
                    println_success!(get_compiled_string!("engine.h1.verbs.recover.recovered_tag"), tag=tag_file.tag_path);
                    skipped = false;
                },
                RecoverResult::DataAlreadyExists => {
                    if !options.batching {
                        println!(get_compiled_string!("engine.h1.verbs.recover.skipped_tag_exists"), tag=tag_file.tag_path);
                    }
                    skipped = true;
                },
                RecoverResult::NoSourceData => {
                    if !options.batching {
                        println_warn!(get_compiled_string!("engine.h1.verbs.recover.skipped_tag_no_source_data"), tag=tag_file.tag_path);
                    }
                    skipped = true;
                },
            }
            drop(l);

            return Ok(!skipped);
        }
    }

    // Can't recover this
    if !options.batching {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.recover.unable_to_recover_tag"), input_group=tag_file.tag_path.get_group())));
    }

    Ok(true)
}

pub fn recover_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args,
                                                       &[],
                                                       &[get_compiled_string!("arguments.specifier.tag_batch_with_group")],
                                                       executable,
                                                       verb.get_description(),
                                                       ArgumentConstraints::new().needs_tags()
                                                                                 .multiple_tags_directories()
                                                                                 .can_overwrite()
                                                                                 .needs_data())?;

    let tag_path = &parsed_args.extra[0];
    let options = RecoverOptions {
        batching: TagFile::uses_batching(tag_path),
        overwrite: parsed_args.named.get("overwrite").is_some(),
        data_dir: Path::new(&parsed_args.named["data"][0]).to_owned()
    };

    Ok(super::do_with_batching_threaded(recover_tag, &tag_path, None, &str_slice_to_path_vec(&parsed_args.named["tags"]), parsed_args.threads, options)?.exit_code())
}
