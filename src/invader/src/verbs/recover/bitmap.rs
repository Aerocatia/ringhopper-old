use strings::*;
use ringhopper::error::*;
use ringhopper::file::TagFile;
use ringhopper::engines::h1::definitions::*;
use ringhopper::engines::h1::*;
use std::convert::TryInto;
use std::path::Path;
use crate::file::*;
use super::RecoverResult;
use std::io::Cursor;

use flate2::{Decompress, Status, FlushDecompress};

use tiff::encoder::*;

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

    // Validate the width and height
    let length = u32::from_be_bytes(tag.compressed_color_plate_data[0..4].try_into().unwrap()) as usize;
    let width = tag.color_plate_width as usize;
    let height = tag.color_plate_height as usize;

    // Check if the width * height * 4 is not equal to length, also accounting for integer overflowing (in which case it would also not be equal).
    if (|| if width.checked_mul(height)?.checked_mul(4)? != length { None } else { Some(()) } )().is_none() {
        return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover.error_bitmap_color_plate_data_invalid")));
    }

    // Try to decompress with deflate
    let mut decompressed_output = Vec::<u8>::new();
    decompressed_output.resize(length, 0);
    let mut decompressor = Decompress::new(true);
    match decompressor.decompress(&tag.compressed_color_plate_data[4..], &mut decompressed_output, FlushDecompress::None).unwrap() {
        Status::StreamEnd => (),
        _ => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover.error_bitmap_color_plate_data_invalid")))
    }

    // Swap red and blue channels
    for i in (0..decompressed_output.len()).step_by(4) {
        let color = &mut decompressed_output[i..i+4];
        let swap = color[0];
        color[0] = color[2];
        color[2] = swap;
    }

    // Encode into a TIFF
    let mut data = Vec::new();
    let mut encoder = TiffEncoder::new(Cursor::new(&mut data)).unwrap();
    let mut image = encoder.new_image::<colortype::RGBA8>(width as u32, height as u32).unwrap();
    image.encoder().write_tag(tiff::tags::Tag::ExtraSamples, &[2u16][..]).unwrap();
    image.rows_per_strip(2).unwrap();

    // Write each strip
    let mut idx = 0;
    while image.next_strip_sample_count() > 0 {
        let sample_count = image.next_strip_sample_count() as usize;
        image.write_strip(&decompressed_output[idx..idx+sample_count]).unwrap();
        idx += sample_count;
    }

    // Done
    image.finish().unwrap();

    // Write
    make_parent_directories(&output_file)?;
    write_file(&output_file, &data)?;

    Ok(RecoverResult::Recovered)
}
