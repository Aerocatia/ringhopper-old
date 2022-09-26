use ringhopper::error::*;
use ringhopper::file::TagFile;
use ringhopper::engines::h1::definitions::*;
use ringhopper::engines::h1::TagFileSerializeFn;
use std::path::Path;
use crate::file::*;
use super::RecoverResult;

pub fn recover_scripts(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult> {
    let tag = Scenario::from_tag_file(tag_data)?.data;

    if tag.source_files.blocks.is_empty() {
        return Ok(RecoverResult::NoSourceData);
    }

    // If the scenario consists of only global_scripts, we should not make a directory.
    //
    // For example, timberland which only has global_scripts but no actual scripts.
    let output_dir = data_dir.join(&tag_file.tag_path.to_string()).parent().unwrap().join("scripts");
    for s in &tag.source_files {
        if s.name.to_str() != "global_scripts" {
            make_directories(&output_dir)?;
        }
    }

    let mut anything_done = false;
    for s in tag.source_files {
        let path = match s.name.to_str() {
            "global_scripts" => data_dir.join("global_scripts.hsc"),
            n => output_dir.join(&(n.to_owned() + ".hsc"))
        };

        // Do not overwrite if not allowed
        if !overwrite && path.is_file() {
            continue
        }

        // Trim null terminator
        let mut data = s.source;
        while data.ends_with(&[0]) {
            data.resize(data.len() - 1, 0);
        };

        // Write it
        write_file(&path, &data)?;
        anything_done = true
    }

    match anything_done {
        true => Ok(RecoverResult::Recovered),
        false => Ok(RecoverResult::DataAlreadyExists)
    }
}
