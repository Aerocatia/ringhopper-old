use ringhopper::error::*;
use ringhopper::file::TagFile;
use ringhopper::engines::h1::definitions::*;
use ringhopper::engines::h1::*;
use std::path::Path;
use crate::file::*;
use super::RecoverResult;

pub fn recover_bitmaps(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult> {
    // Output as a tiff?
    let mut output_file = data_dir.join(&tag_file.tag_path.to_string());
    output_file.set_extension("tif");

    if !overwrite && output_file.is_file() {
        return Ok(RecoverResult::DataAlreadyExists);
    }

    // Parse the tag
    let tag = Bitmap::from_tag_file(tag_data)?.data;

    // No source data (size is less than then length of the length field)
    if tag.compressed_color_plate_data.len() < 4 {
        return Ok(RecoverResult::NoSourceData)
    }

    // Get the width and height
    let width = tag.color_plate_width as usize;
    let height = tag.color_plate_height as usize;

    // Try to decompress with deflate
    let mut decompressed_output = crate::load_bitmap_color_plate(&tag)?;

    // Swap red and blue channels
    for i in (0..decompressed_output.len()).step_by(4) {
        let color = &mut decompressed_output[i..i+4];
        let swap = color[0];
        color[0] = color[2];
        color[2] = swap;
    }

    // Encode into a TIFF
    let data = crate::make_tiff(&decompressed_output, width, height);

    // Write
    make_parent_directories(&output_file)?;
    write_file(&output_file, &data)?;

    Ok(RecoverResult::Recovered)
}
