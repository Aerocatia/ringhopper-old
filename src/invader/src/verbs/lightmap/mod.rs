use ringhopper::bitmap::{ColorPlateBuildBitmap, build_color_plate, BitmapEncoding, ColorPlateOptions, ColorPlateInputType};
use ringhopper::engines::h1::definitions::{Scenario, ScenarioStructureBSP, ScenarioStructureBSPMaterialUncompressedRenderedVertex, ScenarioStructureBSPMaterialCompressedRenderedVertex, ScenarioStructureBSPMaterialCompressedLightmapVertex, ScenarioStructureBSPMaterialUncompressedLightmapVertex, BitmapType, Bitmap, BitmapFormat, BitmapGroupSequence, BitmapData, BitmapDataType, BitmapDataFormat, BitmapUsage};
use ringhopper::types::{ColorARGBInt, String32, Reflexive, TagGroupFn, HALO_DIRECTORY_SEPARATOR};
use ringhopper_proc::*;
use std::num::NonZeroUsize;
use std::path::{PathBuf, Path};
use std::process::ExitCode;
use crate::cmd::*;
use ringhopper::engines::h1::{TagGroup, TagFileSerializeFn, TagSerialize};
use ringhopper::error::ErrorMessage;
use ringhopper::file::TagFile;
use crate::file::*;
use macros::terminal::*;
use ringhopper::error::ErrorMessageResult;
use flate2::{Compress, FlushCompress};

#[derive(Clone)]
struct LightmapOptions {
    fullbright: bool,
    bsp: Option<Vec<String>>,
    tags_dirs: Vec<PathBuf>,
    batching: bool
}

pub fn lightmap_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args,
                                                       &[
                                                           Argument { long: "fullbright", short: 'f', description: "Render lightmaps as fullbright.", parameter: None, multiple: false },
                                                           Argument { long: "bsp", short: 'b', description: "Choose a BSP by name to bake.", parameter: Some("bsp-name"), multiple: true },
                                                       ],
                                                       &[get_compiled_string!("arguments.specifier.tag_batch_without_group")],
                                                       executable,
                                                       verb.get_description(),
                                                       ArgumentConstraints::new().needs_tags()
                                                                                 .multiple_tags_directories()
                                                                                 .uses_threads())?;

    let tag_path = &parsed_args.extra[0];

    let all_tags = str_slice_to_path_vec(&parsed_args.named["tags"]);
    let all_tags = all_tags.iter().map(|f| (*f).to_owned());

    let options = LightmapOptions {
        fullbright: parsed_args.named.contains_key("fullbright"),
        bsp: parsed_args.named.get("bsp").map(|f| f.to_owned()),
        tags_dirs: all_tags.collect(),
        batching: TagFile::uses_batching(tag_path)
    };

    if !options.fullbright {
        return Err(ErrorMessage::StaticString("Lightmapping without --fullbright is not implemented yet"));
    }

    Ok(super::do_with_batching_threaded(lightmap_scenario, &tag_path, Some(TagGroup::Scenario), &str_slice_to_path_vec(&parsed_args.named["tags"]), parsed_args.threads, options)?.exit_code())
}

fn lightmap_scenario(tag_file: &TagFile, log_mutex: super::LogMutex, _: NonZeroUsize, options: &LightmapOptions) -> ErrorMessageResult<bool> {
    let scenario_tag = *Scenario::from_tag_file(&read_file(&tag_file.file_path)?)?.data;

    let tags_dirs: Vec<&Path> = options.tags_dirs.iter().map(|f| f.as_path()).collect();

    let mut bsps_baked = 0;
    for b in scenario_tag.structure_bsps {
        if let Some(base_name) = &options.bsp {
            if !base_name.contains(&b.structure_bsp.get_path_without_extension().rsplit(HALO_DIRECTORY_SEPARATOR).next().unwrap().to_owned()) {
                continue
            }
        }
        bsps_baked += 1;

        let bsp_tag_file = TagFile::from_tag_ref(&tags_dirs, &b.structure_bsp).ok_or_else(|| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.lightmap.error_cannot_find_bsp_tag"), tag=b.structure_bsp)))?;
        if options.fullbright {
            fullbright_bsp_tag(&bsp_tag_file, &log_mutex)?;
        }
    }

    let l = log_mutex.lock();
    if bsps_baked > 0 {
        println_success!(get_compiled_string!("engine.h1.verbs.lightmap.baked"), tag=tag_file.tag_path, bsp=bsps_baked);
    }
    else if !options.batching && options.bsp.is_some() {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.lightmap.baked_skipped"), tag=tag_file.tag_path, bsp=options.bsp.clone().unwrap().join(", "))));
    }
    drop(l);

    Ok(bsps_baked > 0)
}

fn fullbright_bsp_tag(tag_file: &TagFile, log_mutex: &super::LogMutex) -> ErrorMessageResult<()> {
    let mut tag = *ScenarioStructureBSP::from_tag_file(&read_file(&tag_file.file_path)?)?.data;
    tag.lightmaps_bitmap.set_group(TagGroup::Bitmap);
    tag.lightmaps_bitmap.set_path_without_extension(tag_file.tag_path.get_path_without_extension())?;

    // Prepare lightmap vertices lengths
    let uncompressed_lightmap_vertex_len = ScenarioStructureBSPMaterialUncompressedLightmapVertex::tag_size();
    let compressed_lightmap_vertex_len = ScenarioStructureBSPMaterialCompressedLightmapVertex::tag_size();

    let mut last_lightmap_index = 0usize;
    for lmi in 0..tag.lightmaps.blocks.len() {
        let lm = &mut tag.lightmaps[lmi];

        // Some "lightmaps" do not actually contain lightable surfaces
        let mut has_non_transparent_shaders = false;
        for m in &lm.materials {
            match m.shader.get_group() {
                TagGroup::ShaderEnvironment | TagGroup::ShaderModel => {
                    has_non_transparent_shaders = true;
                    break;
                },
                _ => ()
            }
        }

        if !has_non_transparent_shaders {
            continue;
        }

        lm.bitmap = Some(lmi as u16);
        last_lightmap_index = lmi;

        for mati in 0..lm.materials.blocks.len() {
            let mat = &mut lm.materials.blocks[mati];
            let vertex_count = mat.rendered_vertices_count as usize;
            if vertex_count == 0 {
                continue;
            }
            if mat.uncompressed_vertices.is_empty() {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.lightmap.error_uncompressed_vertices_missing"), material=mati, lightmap=lmi)));
            }
            if mat.compressed_vertices.is_empty() {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.lightmap.error_compressed_vertices_missing"), material=mati, lightmap=lmi)));
            }

            // Trim down to size
            let rendered_size_uncompressed = vertex_count * ScenarioStructureBSPMaterialUncompressedRenderedVertex::tag_size();
            let rendered_size_compressed = vertex_count * ScenarioStructureBSPMaterialCompressedRenderedVertex::tag_size();
            if mat.uncompressed_vertices.len() < rendered_size_uncompressed {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.lightmap.error_uncompressed_vertices_corrupt"), material=mati, lightmap=lmi)));
            }
            if mat.compressed_vertices.len() < rendered_size_compressed {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.lightmap.error_compressed_vertices_corrupt"), material=mati, lightmap=lmi)));
            }
            mat.uncompressed_vertices.resize(rendered_size_uncompressed, 0);
            mat.compressed_vertices.resize(rendered_size_compressed, 0);

            // Add in new lightmap vertices
            mat.uncompressed_vertices.resize(vertex_count * uncompressed_lightmap_vertex_len + rendered_size_uncompressed, 0u8);
            mat.compressed_vertices.resize(vertex_count * compressed_lightmap_vertex_len + rendered_size_compressed, 0u8);

            // Set lightmap vertex count to indicate we rendered this
            mat.lightmap_vertices_count = vertex_count as u32;
        }
    }

    // Create the bitmap tag
    let lightmap_count = last_lightmap_index + 1;
    let bitmap_data = vec![ColorARGBInt { a: 255, r: 255, g: 255, b: 255 }; 4*4];
    let bitmap_data_16bit = BitmapEncoding::R5G6B5.encode(&bitmap_data, 4, 4, 1, 1, 0, false);
    let lightmap_bitmap_data = vec![vec![ColorPlateBuildBitmap { width: 4, height: 4, pixel_data: bitmap_data.clone() }; 1]; lightmap_count];
    let (color_plate_pixel_data, color_plate_width, color_plate_height) = build_color_plate(BitmapType::_2dTextures, &lightmap_bitmap_data, true, BitmapEncoding::A8R8G8B8)?;
    let mut options = ColorPlateOptions::default();
    options.input_type = ColorPlateInputType::TwoDimensionalTextures;
    let mut bitmap_tag = Bitmap::default();
    bitmap_tag._type = BitmapType::_2dTextures;
    bitmap_tag.usage = BitmapUsage::LightMap;
    bitmap_tag.encoding_format = BitmapFormat::_16bit;
    bitmap_tag.bitmap_group_sequence.blocks.reserve_exact(lightmap_count);
    bitmap_tag.bitmap_data.blocks.reserve_exact(lightmap_count);

    for l in 0..lightmap_count {
        bitmap_tag.bitmap_group_sequence.blocks.push(BitmapGroupSequence {
            name: String32::from_str(&format!("lightmap_{}", l))?,
            first_bitmap_index: Some(l as u16),
            bitmap_count: 1,
            sprites: Reflexive::default()
        });

        let mut bitmap_data = BitmapData::default();
        bitmap_data.bitmap_class = TagGroup::Bitmap.as_fourcc();
        bitmap_data.width = 4;
        bitmap_data.height = 4;
        bitmap_data.depth = 1;
        bitmap_data._type = BitmapDataType::_2dTexture;
        bitmap_data.format = BitmapDataFormat::R5G6B5;
        bitmap_data.flags.power_of_two_dimensions = true;
        bitmap_data.pixel_data_offset = bitmap_tag.processed_pixel_data.len() as u32;
        bitmap_data.pixel_data_size = bitmap_data_16bit.len() as u32;
        bitmap_tag.bitmap_data.blocks.push(bitmap_data);
        bitmap_tag.processed_pixel_data.extend_from_slice(&bitmap_data_16bit);
    };

    // Try to compress with deflate
    let mut compressor = Compress::new(flate2::Compression::best(), true);
    bitmap_tag.compressed_color_plate_data.extend_from_slice(&(color_plate_pixel_data.len() as u32).to_be_bytes());
    bitmap_tag.compressed_color_plate_data.reserve_exact(color_plate_pixel_data.len() * 2);
    compressor.compress_vec(&color_plate_pixel_data, &mut bitmap_tag.compressed_color_plate_data, FlushCompress::None).unwrap();
    compressor.compress_vec(&[], &mut bitmap_tag.compressed_color_plate_data, FlushCompress::Finish).unwrap();

    // Set the metadata
    bitmap_tag.color_plate_width = color_plate_width as u16;
    bitmap_tag.color_plate_height = color_plate_height as u16;

    // Verify that it's the same!
    debug_assert!(color_plate_pixel_data == crate::load_bitmap_color_plate(&bitmap_tag).unwrap(), "compressed color plate is different!");

    // Write bitmap tag
    let mut bitmap_file_path = tag_file.file_path.clone();
    bitmap_file_path.set_extension("bitmap");
    write_file(&bitmap_file_path, &bitmap_tag.into_tag_file()?)?;

    let l = log_mutex.lock();
    println_success!(get_compiled_string!("engine.h1.verbs.unicode-strings.saved_file"), file=tag.lightmaps_bitmap);
    drop(l);

    // Write BSP tag
    write_file(&tag_file.file_path, &tag.into_tag_file()?)?;

    let l = log_mutex.lock();
    println_success!(get_compiled_string!("engine.h1.verbs.unicode-strings.saved_file"), file=tag_file.tag_path);
    drop(l);

    Ok(())
}
