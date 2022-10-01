use strings::*;
use std::process::ExitCode;
use ringhopper::cmd::*;
use ringhopper::engines::h1::TagGroup;
use ringhopper::error::ErrorMessage;
use ringhopper::file::TagFile;
use crate::file::*;
use macros::terminal::*;
use ringhopper::error::ErrorMessageResult;
use std::path::Path;

mod bitmap;
mod hud_message_text;
mod model;
mod scenario;
mod string_list;

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
    (TagGroup::UnicodeStringList, string_list::recover_unicode_string_list)
];

/// Convert a tag, returning [`None`] if it is impossible to recover a tag with the group,
/// `Some(Err(ErrorMessageResult))` if the tag failed to convert, `Some(Ok(true))` if the tag converted, and
/// `Some(Ok(false))` if the tag did not convert because data is already present.
fn recover_tag(tag_file: &TagFile, file_data: &[u8], data_dir: &Path, overwrite: bool) -> Option<ErrorMessageResult<RecoverResult>> {
    let group = tag_file.tag_path.get_group();
    for fg in RECOVER_FUNCTION_GROUPS {
        if fg.0 == group {
            return Some(fg.1(&file_data, tag_file, data_dir, overwrite));
        }
    }
    None
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

    let overwrite = parsed_args.named.get("overwrite").is_some();

    let tags = match TagFile::from_tag_path_batched(&str_slice_to_path_vec(&parsed_args.named["tags"]), &parsed_args.extra[0], None) {
        Ok(n) => n,
        Err(e) => panic!("{}", e)
    };

    let total = tags.len();
    let mut count = 0usize;
    let mut recovered = 0usize;
    let mut error_count = 0usize;
    let mut convertible = tags.len();
    for i in tags {
        let file_data = read_file(&i.file_path)?;
        match recover_tag(&i, &file_data, str_slice_to_path_vec(&parsed_args.named["data"])[0], overwrite) {
            Some(Ok(result)) => {
                match result {
                    RecoverResult::Recovered => {
                        println_success!(get_compiled_string!("engine.h1.verbs.recover.recovered_tag"), tag=i.tag_path);
                        recovered += 1;
                    },
                    RecoverResult::DataAlreadyExists => println!(get_compiled_string!("engine.h1.verbs.recover.skipped_tag_exists"), tag=i.tag_path),
                    RecoverResult::NoSourceData => println!(get_compiled_string!("engine.h1.verbs.recover.skipped_tag_no_source_data"), tag=i.tag_path)
                };
                count += 1;
            },
            Some(Err(e)) => {
                eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.recover.error_could_not_recover_tag"), tag=i.tag_path, error=e);
                error_count += 1;
            },
            None => {
                if total == 1 {
                    eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.recover.unable_to_recover_tag"), input_group=i.tag_path.get_group());
                    error_count += 1;
                }
                convertible -= 1;
            }
        }
    }

    if convertible == 0 {
        Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover.error_no_tags_found")))
    }
    else if count == 0 {
        Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.recover.error_no_tags_recovered"), error=error_count)))
    }
    else if error_count > 0 {
        println_warn!(get_compiled_string!("engine.h1.verbs.recover.recovered_some_tags_with_errors"), count=recovered, error=error_count);
        Ok(ExitCode::FAILURE)
    }
    else {
        println_success!(get_compiled_string!("engine.h1.verbs.recover.recovered_all_tags"), count=recovered);
        Ok(ExitCode::SUCCESS)
    }
}
