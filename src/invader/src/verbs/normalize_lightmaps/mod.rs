use ringhopper::types::*;
use ringhopper_proc::*;
use std::convert::TryInto;
use std::num::NonZeroUsize;
use std::process::ExitCode;
use crate::cmd::*;
use ringhopper::engines::h1::{TagGroup, TagSerialize};
use ringhopper::engines::h1::definitions::{ScenarioStructureBSP, ScenarioStructureBSPMaterialUncompressedRenderedVertex, ScenarioStructureBSPMaterialUncompressedLightmapVertex};
use ringhopper::engines::h1::TagFileSerializeFn;
use ringhopper::file::TagFile;
use crate::file::*;
use ringhopper::error::ErrorMessageResult;

#[derive(Clone)]
struct NormalizeLightmapsOptions {
    amount: f32
}

// TODO: write a full little endian parser!
fn normalize_tag(tag_file: &TagFile, _: super::LogMutex, _available_threads: NonZeroUsize, options: &NormalizeLightmapsOptions) -> ErrorMessageResult<bool> {
    let mut tag = *ScenarioStructureBSP::from_tag_file(&read_file(&tag_file.file_path)?)?.data;

    for lm in &mut tag.lightmaps {
        for mat in &mut lm.materials {
            let lightmap_vertex_count = mat.lightmap_vertices_count as usize;
            if lightmap_vertex_count == 0 {
                continue
            }
            if mat.uncompressed_vertices.is_empty() {
                return Err(ringhopper::error::ErrorMessage::StaticString(get_compiled_string!("engine.h1.error_improperly_extracted_bsp_vertices_uncompressed")))
            }

            let offset = ScenarioStructureBSPMaterialUncompressedRenderedVertex::tag_size() * (mat.rendered_vertices_count as usize);
            let data_size = ScenarioStructureBSPMaterialUncompressedLightmapVertex::tag_size();

            let length = lightmap_vertex_count * data_size;
            let mut range = &mut mat.uncompressed_vertices[offset..offset+length];

            for _ in 0..lightmap_vertex_count {
                let normal_bytes = &mut range[0..12];

                let (x_bytes, rest_bytes) = normal_bytes.split_at_mut(4);
                let (y_bytes, z_bytes) = rest_bytes.split_at_mut(4);

                let v = Vector3D {
                    x: f32::from_le_bytes(x_bytes.try_into().unwrap()),
                    y: f32::from_le_bytes(y_bytes.try_into().unwrap()),
                    z: f32::from_le_bytes(z_bytes.try_into().unwrap()),
                }
                .normalize()
                .scale(options.amount);

                x_bytes.copy_from_slice(&v.x.to_le_bytes());
                y_bytes.copy_from_slice(&v.y.to_le_bytes());
                z_bytes.copy_from_slice(&v.z.to_le_bytes());

                range = &mut range[data_size..];
            }
        }
    }

    write_file(&tag_file.file_path, &tag.into_tag_file()?)?;
    Ok(true)
}

pub fn normalize_lightmaps_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args,
                                                       &[
                                                           Argument { long: "amount", short: 'a', description: "Normalized scaling (1.0 = 100%)", parameter: Some("amt"), multiple: false },
                                                       ],
                                                       &[get_compiled_string!("arguments.specifier.tag_batch_without_group")],
                                                       executable,
                                                       verb.get_description(),
                                                       ArgumentConstraints::new().needs_tags())?;

    let tag_path = &parsed_args.extra[0];
    let options = NormalizeLightmapsOptions {
        amount: parsed_args.parse_f32("amount")?.unwrap_or(1.0)
    };

    Ok(super::do_with_batching_threaded(normalize_tag, &tag_path, Some(TagGroup::ScenarioStructureBSP), &str_slice_to_path_vec(&parsed_args.named["tags"]), parsed_args.threads, options)?.exit_code())
}
