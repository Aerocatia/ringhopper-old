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
use ringhopper::bitmap::*;

pub fn recover_processed_bitmaps(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, options: &super::RecoverProcessedOptions) -> ErrorMessageResult<RecoverProcessedResult> {
    // Output as a tiff?
    let mut output_file = data_dir.join(&tag_file.tag_path.to_string());
    output_file.set_extension("tif");

    if !options.overwrite && output_file.is_file() {
        return Ok(RecoverProcessedResult::DataAlreadyExists);
    }

    // Parse the tag
    let tag = Bitmap::from_tag_file(tag_data)?.data;

    // Check if source data
    if tag.compressed_color_plate_data.len() >= 4 && !options.force {
        return Ok(RecoverProcessedResult::SourceDataExists)
    }

    // No sequences
    if tag.bitmap_group_sequence.blocks.is_empty() {
        return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_no_sequences")));
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

    if all_bitmaps.is_empty() {
        return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover-processed.error_bitmap_no_bitmaps_recovered")));
    }

    // Crop sprites
    let sprite_spacing = match tag.sprite_spacing {
        0 => match tag.mipmap_count {
            0 => 1,
            _ => 4
        },
        n => n
    } as usize;

    let mut sequences: Vec<Vec<ColorPlateBuildBitmap>> = Vec::with_capacity(sequence_count);
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

                sequence_bitmaps.push(ColorPlateBuildBitmap { width, height, pixel_data });
            }

            sequences.push(sequence_bitmaps);
        }
    }
    else {
        let faces = if tag._type == BitmapType::CubeMaps { 6 } else { 1 };
        for s in 0..sequence_count {
            let seq = &tag.bitmap_group_sequence[s];
            let bitmap_count = seq.bitmap_count as usize;
            if bitmap_count == 0 {
                sequences.push(Vec::new());
                continue;
            }

            let first_bitmap = seq.first_bitmap_index.unwrap() as usize;
            let texture_count = match tag._type {
                BitmapType::CubeMaps => faces,
                BitmapType::_2dTextures | BitmapType::InterfaceBitmaps => bitmap_count,
                BitmapType::_3dTextures => tag.bitmap_data[first_bitmap].depth as usize,
                _ => unreachable!()
            };

            let mut sequence_bitmaps = Vec::with_capacity(texture_count);

            let textures_to_read = if tag._type == BitmapType::CubeMaps || tag._type == BitmapType::_3dTextures {
                texture_count
            }
            else {
                1
            };

            for i in first_bitmap..first_bitmap + bitmap_count {
                let bitmap = &tag.bitmap_data[i];
                let width = bitmap.width as usize;
                let height = bitmap.height as usize;
                let pixel_count = width * height;
                let bitmap_data = &all_bitmaps[&i];
                for o in (0..pixel_count * textures_to_read).step_by(pixel_count) {
                    sequence_bitmaps.push(ColorPlateBuildBitmap { width, height, pixel_data: bitmap_data[o..o+pixel_count].to_owned() })
                }
            }

            sequences.push(sequence_bitmaps);
        }
    }

    // Build
    let (input_data, width, height) = build_color_plate(tag._type, &sequences, options.force_plate, BitmapEncoding::A8B8G8R8)?;

    // Encode into a TIFF
    let data = crate::make_tiff(&input_data, width, height);

    // Write
    make_parent_directories(&output_file)?;
    write_file(&output_file, &data)?;

    Ok(RecoverProcessedResult::Recovered)
}
