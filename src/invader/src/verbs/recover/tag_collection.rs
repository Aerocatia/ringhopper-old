use ringhopper::error::*;
use ringhopper::file::TagFile;
use ringhopper::engines::h1::definitions::*;
use ringhopper::engines::h1::TagFileSerializeFn;
use std::path::Path;
use crate::file::*;
use super::RecoverResult;

macro_rules! recover_string_list (
    ($parser:ty, $tag_data:expr, $tag_file:expr, $data_dir:expr, $overwrite:expr) => {{
        let mut output = $data_dir.join(&$tag_file.tag_path.to_string());
        output.set_extension("txt");
        if !$overwrite && output.is_file() {
            return Ok(RecoverResult::DataAlreadyExists);
        }

        let tag = *<$parser>::from_tag_file($tag_data)?.data;
        let mut output_data = Vec::new();
        for tag in tag.tags {
            output_data.extend_from_slice(tag.reference.get_path_with_extension().as_bytes());
            output_data.extend_from_slice(&['\r' as u8, '\n' as u8]);
        }
        make_parent_directories(&output)?;
        write_file(&output, &output_data)?;
        Ok(RecoverResult::Recovered)
    }}
);

pub fn recover_ui_widget_collection(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult> {
    recover_string_list!(UIWidgetCollection, tag_data, tag_file, data_dir, overwrite)
}

pub fn recover_tag_collection(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult> {
    recover_string_list!(TagCollection, tag_data, tag_file, data_dir, overwrite)
}
