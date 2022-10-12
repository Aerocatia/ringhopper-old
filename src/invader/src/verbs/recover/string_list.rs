use ringhopper::error::*;
use ringhopper::file::TagFile;
use ringhopper::engines::h1::definitions::*;
use ringhopper::engines::h1::TagFileSerializeFn;
use strings::get_compiled_string;
use std::path::Path;
use crate::file::*;
use crate::string::*;
use super::RecoverResult;

macro_rules! recover_string_list (
    ($end_string:expr, $parser:ty, $tag_data:expr, $tag_file:expr, $data_dir:expr, $overwrite:expr, $output_data:expr, $cleanup_slice_fn:expr) => {{
        let mut output = $data_dir.join(&$tag_file.tag_path.to_string());
        output.set_extension("txt");
        if !$overwrite && output.is_file() {
            return Ok(RecoverResult::DataAlreadyExists);
        }

        // Add each string, looking for any null terminators in case of improper editing.
        for s in <$parser>::from_tag_file($tag_data)?.data.strings {
            $output_data.extend_from_slice($cleanup_slice_fn(&s.string)?);
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
    recover_string_list!(end_string, StringList, tag_data, tag_file, data_dir, overwrite, output_data, |s| Ok(to_terminator(s, 0)) )
}

pub fn recover_unicode_string_list(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult> {
    let end_string = {
        let mut data_u8: Vec<u8> = Vec::new();
        for i in "\r\n###END-STRING###\r\n".as_bytes() {
            data_u8.push(*i);
            data_u8.push(0);
        }
        data_u8
    };

    // Check if the byte array is divisible by 2, then search for a null terminator
    fn terminate_utf16(s: &[u8]) -> ErrorMessageResult<&[u8]> {
        if s.len() % 2 != 0 {
            return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover.error_hud_messages_not_valid_utf16")));
        }
        for i in (0..s.len()).step_by(2) {
            if s[i] == 0 && s[i + 1] == 0 {
                return Ok(&s[..i]);
            }
        }
        Ok(s)
    }

    let mut output_data = vec![0xFF, 0xFE]; // LE BOM
    recover_string_list!(&end_string, UnicodeStringList, tag_data, tag_file, data_dir, overwrite, output_data, terminate_utf16)
}
