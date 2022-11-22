use ringhopper::error::*;
use ringhopper::file::TagFile;
use ringhopper::engines::h1::definitions::*;
use ringhopper::engines::h1::*;
use ringhopper::types::ColorARGBInt;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::path::Path;
use crate::file::*;
use ringhopper_proc::*;
use super::RecoverProcessedResult;
use ringhopper::bitmap::BitmapEncoding;

pub fn recover_processed_bitmaps(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool, force: bool) -> ErrorMessageResult<RecoverProcessedResult> {
    // Output as a tiff?
    let mut output_file = data_dir.join(&tag_file.tag_path.to_string());
    output_file.set_extension("tif");

    if !overwrite && output_file.is_file() {
        return Ok(RecoverProcessedResult::DataAlreadyExists);
    }

    // Parse the tag
    let tag = Bitmap::from_tag_file(tag_data)?.data;

    // Check if source data
    if tag.compressed_color_plate_data.len() >= 4 && !force {
        return Ok(RecoverProcessedResult::SourceDataExists)
    }

    // Make sure sprites can be done
    if tag._type == BitmapType::Sprites {
        let mut has_sprites = false;
        for s in &tag.bitmap_group_sequence {
            if !s.sprites.blocks.is_empty() {
                has_sprites = true;
                break;
            }
        }
        if !has_sprites {
            return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_bad_sprite_empty")));
        }
        for b in &tag.bitmap_data {
            if b._type != BitmapDataType::_2dTexture {
                return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_bad_sprite_type")));
            }
        }
    }

    // Make sure types are correct
    let mut is_multitexture = false;
    for b in &tag.bitmap_data {
        if b._type != BitmapDataType::_3dTexture && b.depth != 1 {
            return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_bad_depth")));
        }
        match b._type {
            BitmapDataType::CubeMap => if tag._type != BitmapType::CubeMaps {
                return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_bad_cubemaps")));
            }
            else {
                is_multitexture = true;
            },
            BitmapDataType::_3dTexture => if tag._type != BitmapType::_3dTextures {
                return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_bad_3d")));
            }
            else {
                is_multitexture = true;
            },
            BitmapDataType::_2dTexture => if tag._type != BitmapType::_2dTextures && tag._type != BitmapType::Sprites && tag._type != BitmapType::InterfaceBitmaps {
                return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_bad_2d")));
            }
            else {
                is_multitexture = false;
            },
            BitmapDataType::White => {
                return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_bad_white_type")));
            }
        }
    }
    if is_multitexture {
        for s in &tag.bitmap_group_sequence {
            if s.first_bitmap_index.is_some() && s.bitmap_count > 1 {
                return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_bad_multitex")));
            }
        }
    }

    let mut all_bitmaps: BTreeMap<usize, Vec<ColorARGBInt>> = BTreeMap::new();

    fn decode_bitmap(tag: &Bitmap, all_bitmaps: &mut BTreeMap<usize, Vec<ColorARGBInt>>, index: usize) -> ErrorMessageResult<()> {
        if all_bitmaps.contains_key(&index) {
            return Ok(());
        }
        let bitmap = &tag.bitmap_data[index];
        let encoding: BitmapEncoding = bitmap.format.try_into()?;

        let faces = match bitmap._type { BitmapDataType::CubeMap => 6, _ => 1 };
        let width = bitmap.width as usize;
        let height = bitmap.height as usize;
        let depth = bitmap.depth.max(1) as usize;
        let mipmaps = bitmap.mipmap_count as usize;

        let start = bitmap.pixel_data_offset as usize;
        let end = start.saturating_add(encoding.calculate_size_of_texture(width, height, depth, faces, mipmaps));

        let input = tag.processed_pixel_data.get(start..end).ok_or_else(|| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_out_of_bounds"), bitmap=index)))?;
        let base_texture_size = encoding.calculate_size_of_texture(width, height, depth, faces, 0);

        all_bitmaps.insert(
            index,
            encoding.decode(
                &input[0..base_texture_size],
                width as usize,
                height as usize,
                depth as usize,
                faces,
                0 // 0 mipmaps as we only need the base map
            )
        );

        Ok(())
    }

    let bitmap_count = tag.bitmap_data.blocks.len();
    let sequence_count = tag.bitmap_group_sequence.blocks.len();

    if tag._type == BitmapType::Sprites {
        for s in 0..sequence_count {
            let seq = &tag.bitmap_group_sequence[s];
            for spr in &seq.sprites {
                let q = spr.bitmap_index.unwrap_or(0xFFFF) as usize;
                if q >= bitmap_count {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_sequence_invalid_bitmap_index"), sequence=s, index=q, count=bitmap_count)));
                }
                decode_bitmap(&tag, &mut all_bitmaps, q)?;
            }
        }
    }
    else {
        for s in 0..sequence_count {
            let seq = &tag.bitmap_group_sequence[s];
            if seq.first_bitmap_index.is_some() && seq.bitmap_count > 0 {
                let first = seq.first_bitmap_index.unwrap() as usize;
                let end = first + seq.bitmap_count as usize;
                for q in first..end {
                    if q >= bitmap_count {
                        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_sequence_invalid_bitmap_index"), sequence=s, index=q, count=bitmap_count)));
                    }
                    decode_bitmap(&tag, &mut all_bitmaps, q)?;
                }
            }
        }
    }

    // Crop sprites
    struct ExtractedImage {
        width: usize,
        height: usize,
        pixel_data: Vec<ColorARGBInt>
    }

    let sprite_spacing = match tag.sprite_spacing {
        0 => match tag.mipmap_count {
            0 => 1,
            _ => 4
        },
        n => n
    } as usize;

    let mut bitmaps_to_put_in_color_plate: Vec<Vec<ExtractedImage>> = Vec::with_capacity(sequence_count);
    if tag._type == BitmapType::Sprites {
        for s in 0..sequence_count {
            let seq = &tag.bitmap_group_sequence[s];
            let sprite_count = seq.sprites.blocks.len();
            let mut sequence_bitmaps = Vec::with_capacity(sprite_count);

            for spr in &seq.sprites {
                let q = spr.bitmap_index.unwrap_or(0xFFFF) as usize;
                let b = &tag.bitmap_data[q];
                let original_pixel_data = &all_bitmaps[&q];

                if spr.left > spr.right || spr.top > spr.bottom || spr.left < 0.0 || spr.right > 1.0 || spr.top < 0.0 || spr.bottom > 1.0 {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_corrupted_invalid_sprite_dimensions"), sequence=s)));
                }

                let mut sprites_on_this_bitmap = 0;
                for seq2 in &tag.bitmap_group_sequence {
                    for spr2 in &seq2.sprites {
                        if spr2.bitmap_index == spr.bitmap_index {
                            sprites_on_this_bitmap += 1;
                        }
                    }
                }

                let spacing = match sprites_on_this_bitmap {
                    1 => 0,
                    _ => sprite_spacing
                };

                let x1 = ((spr.left as f64 * b.width as f64).round() as usize).saturating_add(spacing);
                let y1 = ((spr.top as f64 * b.height as f64).round() as usize).saturating_add(spacing);
                let x2 = ((spr.right as f64 * b.width as f64).round() as usize).saturating_sub(spacing);
                let y2 = ((spr.bottom as f64 * b.height as f64).round() as usize).saturating_sub(spacing);

                if x1 >= x2 || y1 >= y2 || x2 > b.width as usize || y2 > b.width as usize {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_corrupted_invalid_sprite_dimensions"), sequence=s)));
                }

                let width = x2 - x1;
                let height = y2 - y1;

                let mut pixel_data = Vec::with_capacity(width * height);
                for y in y1..y2 {
                    pixel_data.extend_from_slice(&original_pixel_data[y * b.width as usize..][x1..x2]);
                }

                sequence_bitmaps.push(ExtractedImage { width, height, pixel_data });
            }

            bitmaps_to_put_in_color_plate.push(sequence_bitmaps);
        }
    }
    else {
        let faces = if tag._type == BitmapType::CubeMaps { 6 } else { 1 };
        for s in 0..sequence_count {
            let seq = &tag.bitmap_group_sequence[s];
            let bitmap_count = seq.bitmap_count as usize;
            if bitmap_count == 0 {
                bitmaps_to_put_in_color_plate.push(Vec::new());
                continue;
            }

            let first_bitmap = seq.first_bitmap_index.unwrap() as usize;
            let texture_count = match tag._type {
                BitmapType::CubeMaps => faces,
                BitmapType::_2dTextures => bitmap_count,
                BitmapType::_3dTextures => tag.bitmap_data[first_bitmap].depth as usize,
                _ => unreachable!()
            };

            let mut sequence_bitmaps = Vec::with_capacity(texture_count);

            for i in first_bitmap..first_bitmap + bitmap_count {
                let bitmap = &tag.bitmap_data[i];
                let width = bitmap.width as usize;
                let height = bitmap.height as usize;
                let pixel_count = width * height;
                let bitmap_data = &all_bitmaps[&i];
                for o in (0..pixel_count * texture_count).step_by(pixel_count) {
                    sequence_bitmaps.push(ExtractedImage { width, height, pixel_data: bitmap_data[o..o+pixel_count].to_owned() })
                }
            }

            bitmaps_to_put_in_color_plate.push(sequence_bitmaps);
        }
    }

    let contains_color = |color: ColorARGBInt| -> bool {
        for s in &bitmaps_to_put_in_color_plate {
            for b in s {
                for p in &b.pixel_data {
                    if p.same_color(color) {
                        return true;
                    }
                }
            }
        }
        return false;
    };

    let blue = ColorARGBInt { a: 255, r: 0, g: 0, b: 255 };
    let magenta = ColorARGBInt { a: 255, r: 255, g: 0, b: 255 };

    let background = match contains_color(blue) {
        false => blue,
        true => {
            let mut color_to_find = None;
            for i in 0xFF000000u32..=0xFFFFFFFFu32 {
                let color = ColorARGBInt::from_a8r8g8b8(i);
                if !contains_color(color) {
                    color_to_find = Some(color);
                    break;
                }
            }
            color_to_find.ok_or(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_no_unique_background")))?
        }
    };

    let divider = match contains_color(magenta) {
        false => magenta,
        true => {
            let mut color_to_find = None;
            for i in 0xFF000000u32..=0xFFFFFFFFu32 {
                let color = ColorARGBInt::from_a8r8g8b8(i);
                if color == background {
                    continue;
                }
                if !contains_color(color) {
                    color_to_find = Some(color);
                    break;
                }
            }
            color_to_find.ok_or(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_no_unique_divider")))?
        }
    };

    let mut height;
    let mut width;
    let input_data: Vec<u8>;

    let single_bitmap = sequence_count == 1 && bitmap_count == 1;

    // Make single 2D texture
    if single_bitmap && tag._type == BitmapType::_2dTextures {
        let b = &bitmaps_to_put_in_color_plate[0][0];
        height = b.height;
        width = b.width;
        input_data = BitmapEncoding::A8B8G8R8.encode(&b.pixel_data, width, height, 1, 1, 0, false);
    }

    // Make unrolled cubemap
    else if single_bitmap && tag._type == BitmapType::CubeMaps && (bitmaps_to_put_in_color_plate[0][0].width == bitmaps_to_put_in_color_plate[0][0].height) {
        let b = &bitmaps_to_put_in_color_plate[0][0];
        let length = b.width;

        height = length * 3;
        width = length * 4;

        let rotate_0 = |offset_x, offset_y, _| (offset_x, offset_y);
        let rotate_90 = |offset_x, offset_y, length| (length - (offset_y + 1), offset_x);
        let rotate_180 = |offset_x, offset_y, length| (length - (offset_x + 1), length - (offset_y + 1));
        let rotate_270 = |offset_x, offset_y, length| (offset_y, length - (offset_x + 1));

        let read_face_and_rotate = |index: usize, output: &mut [ColorARGBInt], to_x: usize, to_y: usize, get_pixel: fn (offset_x: usize, offset_y: usize, length: usize) -> (usize, usize)| {
            let pixel_data = &bitmaps_to_put_in_color_plate[0][index].pixel_data;

            for y in 0..length {
                for x in 0..length {
                    let (rx, ry) = get_pixel(x, y, length);
                    output[x + to_x + (y + to_y) * width] = pixel_data[rx + ry * length];
                }
            }
        };

        let mut pixel_data = vec![ColorARGBInt { a: 255, r: 0, g: 0, b: 0}; width * height];

        read_face_and_rotate(0, &mut pixel_data, length * 0, length * 1, rotate_270);
        read_face_and_rotate(1, &mut pixel_data, length * 1, length * 1, rotate_180);
        read_face_and_rotate(2, &mut pixel_data, length * 2, length * 1, rotate_90);
        read_face_and_rotate(3, &mut pixel_data, length * 3, length * 1, rotate_0);
        read_face_and_rotate(4, &mut pixel_data, length * 0, length * 0, rotate_270);
        read_face_and_rotate(5, &mut pixel_data, length * 0, length * 2, rotate_270);

        input_data = BitmapEncoding::A8B8G8R8.encode(&pixel_data, width, height, 1, 1, 0, false);
    }

    // Make a color plate
    else {
        height = 1usize; // start with 1 for the key
        width = 3usize; // start with 3 for the sequence divider

        for s in 0..sequence_count {
            height += 2; // add 2 for the sequence divider and padding

            let mut sequence_width = 1usize; // start with 1 because you need some pixel on the left
            let mut sequence_height = 0usize;

            for b in &bitmaps_to_put_in_color_plate[s] {
                sequence_width += b.width + 1;
                sequence_height = b.height.max(sequence_height);
            }

            width = width.max(sequence_width);
            height += sequence_height + 1; // add 1 for padding on the bottom
        }

        // Make the color plate
        let mut pixels = vec![background; width * height];
        pixels[1] = divider;

        let mut y = 1;
        for s in 0..sequence_count {
            for x in 0..width {
                pixels[y * width + x] = divider;
            }

            y += 2;

            let mut x = 1;
            let mut sequence_height = 0usize;
            for b in &bitmaps_to_put_in_color_plate[s] {
                for y_sub in 0..b.height {
                    for x_sub in 0..b.width {
                        pixels[x + x_sub + (y + y_sub) * width] = b.pixel_data[x_sub + y_sub * b.width];
                    }
                }
                x += 1 + b.width;
                sequence_height = b.height.max(sequence_height);
            }

            y += sequence_height + 1;
        }

        // Encode into A8B8R8G8
        input_data = BitmapEncoding::A8B8G8R8.encode(&pixels, width, height, 1, 1, 0, false);
    }

    // Encode into a TIFF
    let data = crate::make_tiff(&input_data, width, height);

    // Write
    make_parent_directories(&output_file)?;
    write_file(&output_file, &data)?;

    Ok(RecoverProcessedResult::Recovered)
}
