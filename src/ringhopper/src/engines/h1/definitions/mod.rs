//! Halo: Combat Evolved tag definitions.

extern crate h1_code_generator;
use self::h1_code_generator::load_json_def;
use crate::engines::h1::{TagSerialize, TagSerializeSwapped, TagFileSerializeFn, TagReference, ScenarioScriptNodeValue, Index, TagID, Pointer, TAG_FILE_HEADER_LEN, TagGroup, ParsedTagFile, TagFileHeader};
use crate::error::*;
use crate::types::*;
use crate::types::tag::{TagBlockFn, TagField, TagGroupFn};
use strings::*;

use std::convert::{TryFrom, From};

load_json_def!();

// Convert model geometry parts to gbxmodel
impl From<ModelGeometryPart> for GBXModelGeometryPart {
    fn from(part: ModelGeometryPart) -> Self {
        GBXModelGeometryPart {
            base_struct: part,
            local_node_count: 0,
            local_node_indices: [0u8; 22]
        }
    }
}

// Convert gbxmodel geometry parts to model (does not take into account local nodes)
impl From<GBXModelGeometryPart> for ModelGeometryPart {
    fn from(part: GBXModelGeometryPart) -> Self {
        part.base_struct
    }
}

macro_rules! convert_model_geometry {
    ($from:tt, $to:tt) => {
        impl From<$from> for $to {
            fn from(geo: $from) -> Self {
                let mut parts = Vec::new();
                parts.reserve(geo.parts.blocks.len());
                for i in geo.parts.blocks {
                    parts.push(i.into());
                }
                $to { flags: geo.flags, parts: Reflexive::new(parts) }
            }
        }
    }
}
convert_model_geometry!(ModelGeometry, GBXModelGeometry);
convert_model_geometry!(GBXModelGeometry, ModelGeometry);

macro_rules! copy_model {
    ($from_obj:tt, $to_type:tt) => {
        $to_type {
            flags: $from_obj.flags,
            node_list_checksum: $from_obj.node_list_checksum,

            super_high_detail_cutoff: $from_obj.super_high_detail_cutoff,
            high_detail_cutoff: $from_obj.high_detail_cutoff,
            medium_detail_cutoff: $from_obj.medium_detail_cutoff,
            low_detail_cutoff: $from_obj.low_detail_cutoff,
            super_low_detail_cutoff: $from_obj.super_low_detail_cutoff,

            super_high_detail_node_count: $from_obj.super_high_detail_node_count,
            high_detail_node_count: $from_obj.high_detail_node_count,
            medium_detail_node_count: $from_obj.medium_detail_node_count,
            low_detail_node_count: $from_obj.low_detail_node_count,
            super_low_detail_node_count: $from_obj.super_low_detail_node_count,

            base_map_u_scale: $from_obj.base_map_u_scale,
            base_map_v_scale: $from_obj.base_map_v_scale,

            markers: $from_obj.markers,
            nodes: $from_obj.nodes,
            geometries: {
                let mut geometries = Vec::new();
                geometries.reserve($from_obj.geometries.blocks.len());
                for i in $from_obj.geometries.blocks {
                    geometries.push(i.into());
                }
                Reflexive::new(geometries)
            },
            regions: $from_obj.regions,
            shaders: $from_obj.shaders,
        }
    }
}

impl TryFrom<GBXModel> for Model {
    type Error = ErrorMessage;

    fn try_from(model: GBXModel) -> ErrorMessageResult<Self> {
        let mut model = model;

        if model.flags.parts_have_local_nodes {
            model.flags.parts_have_local_nodes = false;

            for g in &mut model.geometries {
                for p in &mut g.parts {
                    let all_indices = match p.local_node_indices.get(0..p.local_node_count as usize) {
                        Some(n) => n,
                        None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.gbxmodel.error_invalid_local_nodes")))
                    };

                    let convert_index = |index: Index| {
                        if index.is_none() {
                            return Ok(None) // null indices are passed through
                        };

                        match all_indices.get(index.unwrap() as usize) {
                            Some(&n) => Ok(Some(n as u16)),
                            None => Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.gbxmodel.error_invalid_local_nodes")))
                        }
                    };

                    for n in &mut p.base_struct.uncompressed_vertices {
                        n.node0_index = convert_index(n.node0_index)?;
                        n.node1_index = convert_index(n.node1_index)?;
                    }
                }
            }
        }

        Ok(copy_model!(model, Model))
    }
}

impl From<Model> for GBXModel {
    fn from(model: Model) -> Self {
        copy_model!(model, GBXModel)
    }
}

macro_rules! shader_transparent_chicago_conversion {
    ($from:tt, $to:tt, $maps_from:tt, $maps_to:tt, $flags_from:tt, $flags_to:tt) => {
        impl From<$from> for $to {
            fn from(shader: $from) -> Self {
                let mut shader_to = $to::default();

                shader_to.base_struct = shader.base_struct;
                shader_to.numeric_counter_limit = shader.numeric_counter_limit;
                shader_to.first_map_type = shader.first_map_type;
                shader_to.$flags_to = shader.$flags_from;
                shader_to.framebuffer_blend_function = shader.framebuffer_blend_function;
                shader_to.framebuffer_fade_mode = shader.framebuffer_fade_mode;
                shader_to.framebuffer_fade_source = shader.framebuffer_fade_source;
                shader_to.lens_flare_spacing = shader.lens_flare_spacing;
                shader_to.lens_flare = shader.lens_flare;
                shader_to.extra_layers = shader.extra_layers;
                shader_to.$maps_to = shader.$maps_from;
                shader_to.extra_flags = shader.extra_flags;

                shader_to
            }
        }
    }
}

shader_transparent_chicago_conversion!(
    ShaderTransparentChicagoExtended,
    ShaderTransparentChicago,
    maps_4_stage,
    maps,
    shader_transparent_chicago_extended_flags,
    shader_transparent_chicago_flags
);

shader_transparent_chicago_conversion!(
    ShaderTransparentChicago,
    ShaderTransparentChicagoExtended,
    maps,
    maps_4_stage,
    shader_transparent_chicago_flags,
    shader_transparent_chicago_extended_flags
);
