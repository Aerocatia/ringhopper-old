use ringhopper::error::*;
use ringhopper::file::TagFile;
use ringhopper::engines::h1::definitions::*;
use ringhopper::engines::h1::TagFileSerializeFn;
use std::path::Path;
use crate::file::*;
use super::RecoverResult;

macro_rules! recover_string_list (
    ($end_string:expr, $parser:ty, $tag_data:expr, $tag_file:expr, $data_dir:expr, $overwrite:expr, $output_data:expr) => {{
        let mut output = $data_dir.join(&$tag_file.tag_path.to_string());
        output.set_extension("txt");
        if !$overwrite && output.is_file() {
            return Ok(RecoverResult::DataAlreadyExists);
        }
        let tag = <$parser>::from_tag_file($tag_data)?.data;
        for s in tag.strings {
            $output_data.extend_from_slice(&s.string);
            $output_data.extend_from_slice($end_string);
        }
        make_parent_directories(&output)?;
        write_file(&output, &$output_data)?;
        Ok(RecoverResult::Recovered)
    }}
);

pub fn recover_string_list(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult> {
    let end_string = "\r\n###END-STRING###\r\n".as_bytes();
    let mut output_data = Vec::new();
    recover_string_list!(end_string, StringList, tag_data, tag_file, data_dir, overwrite, output_data)
}

pub fn recover_unicode_string_list(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult> {
    let end_string = {
        let data: Vec<u16> = "\r\n###END-STRING###\r\n".encode_utf16().collect();
        let mut data_u8: Vec<u8> = Vec::new();

        for i in data {
            data_u8.push((i & 0xFF) as u8);
            data_u8.push(((i & 0xFF00) >> 8) as u8);
        }

        data_u8
    };
    let mut output_data = vec![0xFF, 0xFE]; // LE BOM
    recover_string_list!(&end_string, UnicodeStringList, tag_data, tag_file, data_dir, overwrite, output_data)
}
