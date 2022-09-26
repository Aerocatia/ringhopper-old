use strings::*;
use ringhopper::error::*;
use ringhopper::file::TagFile;
use ringhopper::engines::h1::definitions::*;
use ringhopper::engines::{h1::*, HUD_MESSAGE_ELEMENT_TYPES};
use std::path::Path;
use crate::file::*;
use super::RecoverResult;

pub fn recover_hud_messages(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult> {
    let mut output_file = data_dir.join(&tag_file.tag_path.to_string());
    output_file.set_extension("hmt");

    if !overwrite && output_file.is_file() {
        return Ok(RecoverResult::DataAlreadyExists);
    }

    let tag = HUDMessageText::from_tag_file(tag_data)?.data;
    let mut output_data: Vec<u16> = vec![0xFEFF]; // bom

    let data_buffer_len = tag.text_data.len();
    if data_buffer_len % 2 != 0 {
        return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover.error_hud_messages_not_valid_utf16")))
    }

    // Convert u8 to u16
    let text_data = {
        let mut vec = Vec::<u16>::new();
        vec.reserve(data_buffer_len / 2);
        for i in (0..data_buffer_len).step_by(2) {
            vec.push(((tag.text_data[i] as u16)) | ((tag.text_data[i + 1] as u16) << 8));
        }
        vec
    };

    // Go through each message
    let element_count = tag.message_elements.blocks.len();
    for i in 0..tag.messages.blocks.len() {
        let message = &tag.messages[i];
        output_data.extend(format!("{}=", message.name.to_str()).encode_utf16());

        let first_index = message.start_index_of_message_block as usize;
        let count = message.panel_count as usize;
        let end_index = first_index + count;
        if end_index > element_count {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.recover.error_hud_messages_out_of_bounds_message"), message_index=i, message_name=message.name.to_str())));
        }

        // Go through each element
        let mut cursor = message.start_index_into_text_blob as usize;
        for e in first_index..end_index {
            let element = &tag.message_elements[e];

            match element._type {
                0 => {
                    let end = match cursor.checked_add(element.data as usize) {
                        Some(n) => n,
                        None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.serialize.error_architecture_limit_exceeded")))
                    };
                    let input = match text_data.get(cursor..end) {
                        // go up to the first null terminator
                        Some(n) => {
                            (|| {
                                for i in 0..n.len() {
                                    if n[i] == 0 {
                                        return &n[0..i];
                                    }
                                }
                                n
                            })()
                        },
                        None => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.recover.error_hud_messages_out_of_bounds_element"), message_index=i)))
                    };
                    output_data.extend_from_slice(input);
                    cursor = end;
                },
                1 => match HUD_MESSAGE_ELEMENT_TYPES.get(element.data as usize) {
                    Some(n) => {
                        output_data.push('%' as u16);
                        output_data.extend(n.encode_utf16());
                    },
                    None => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.recover.error_hud_messages_out_of_bounds_element"), message_index=i)))
                },
                _ => unreachable!()
            };
        }

        output_data.extend("\r\n".encode_utf16());
    }

    make_parent_directories(&output_file)?;

    let mut data = Vec::new();
    data.reserve(output_data.len() * 2);

    // UTF-16 (LE) to bytes
    for i in output_data {
        data.push((i & 0xFF) as u8);
        data.push(((i & 0xFF00) >> 8) as u8);
    }
    write_file(&output_file, &data)?;

    Ok(RecoverResult::Recovered)
}
