use ringhopper::error::*;
use ringhopper::file::TagFile;
use ringhopper::engines::h1::definitions::*;
use ringhopper::engines::h1::TagFileSerializeFn;
use ringhopper::engines::h1::jms::*;
use ringhopper::engines::h1::*;
use strings::*;
use ringhopper::types::*;
use std::path::Path;
use crate::file::*;
use std::convert::TryFrom;
use std::collections::HashMap;
use std::convert::TryInto;
use super::RecoverResult;

macro_rules! convert_try_into_error {
    ($input:expr) => {
        ($input).map_err(|e| ErrorMessage::AllocatedString(format!("{e}")))
    }
}

fn make_jms(model: &Model, permutation: &str, lod: usize) -> ErrorMessageResult<Option<JMS>> {
    // Set the checksum value
    let node_list_checksum = model.node_list_checksum;

    // Fill out the nodes
    let mut nodes = Vec::<Node>::new();
    nodes.reserve(model.nodes.blocks.len());
    for node in &model.nodes {
        nodes.push(Node {
            name: node.name.to_str().to_owned(),
            position: node.default_translation,
            rotation: node.default_rotation,
            sibling_node: node.next_sibling_node_index,
            first_child: node.first_child_node_index
        })
    }

    // Have a thing for regions too
    let mut regions = Vec::<Region>::new();
    regions.reserve(model.regions.blocks.len());

    // Markers!
    let mut markers = Vec::<Marker>::new();

    // Map a shader indices from the model to the JMS
    let mut shader_map: HashMap<String, Option<usize>> = HashMap::new();

    // Materials!
    let mut materials = Vec::<Material>::new();

    let u_scale = if model.base_map_u_scale == 0.0 { 1.0 } else { model.base_map_u_scale };
    let v_scale = if model.base_map_v_scale == 0.0 { 1.0 } else { model.base_map_v_scale };

    // Vertices!
    let mut vertices = Vec::<Vertex>::new();

    // Triangles!
    let mut triangles = Vec::<Triangle>::new();

    // Get the region
    for r in &model.regions {
        for p in &r.permutations {
            if permutation == p.name.to_str() {
                // Get the geometry index. Ensure it's not the same as a higher LOD which we would've exported anyway.
                let geometry_index;
                let can_be_used;
                match lod {
                    0 => {
                        geometry_index = p.super_high;
                        can_be_used = true;
                    }
                    1 => {
                        geometry_index = p.high;
                        can_be_used = geometry_index != p.super_high;
                    }
                    2 => {
                        geometry_index = p.medium;
                        can_be_used = geometry_index != p.high;
                    }
                    3 => {
                        geometry_index = p.low;
                        can_be_used = geometry_index != p.medium;
                    }
                    4 => {
                        geometry_index = p.super_low;
                        can_be_used = geometry_index != p.low;
                    }
                    _ => unreachable!()
                }

                // Get the geometry
                let geometry = match model.geometries.try_get_with_index(geometry_index)? {
                    Some(n) => n,
                    None => continue
                };

                // Skip it?
                if !can_be_used {
                    continue;
                }

                // Add our new region
                let new_region_index = regions.len();
                regions.push(Region {
                    name: r.name.to_str().to_owned()
                });

                // If superhigh, export markers too
                if lod == 0 {
                    for m in &p.markers {
                        markers.push(Marker {
                            name: m.name.to_str().to_owned(),
                            node: m.node_index,
                            position: m.translation,
                            rotation: m.rotation,
                            region: Some(new_region_index as u16),
                            radius: 1.0
                        });
                    }
                }

                for p in &geometry.parts {
                    // Is the shader valid?
                    let shader_index = p.shader_index.try_unwrap_index()?;
                    if shader_index >= model.shaders.blocks.len() {
                        panic!()
                    }

                    // Here we go
                    let shader_entry = &model.shaders[shader_index];
                    let shader_path = shader_entry.shader.get_path_without_extension();
                    let mut shader_name = match shader_path.rfind(HALO_DIRECTORY_SEPARATOR) {
                        Some(n) => shader_path.split_at(n+1).1.to_owned(),
                        None => shader_path.to_owned()
                    };

                    // If the permutation is non-zero, also add the permutation index
                    let permutation = shader_entry.permutation.unwrap_or(0);
                    if shader_entry.permutation.unwrap_or(0) > 0 {
                        shader_name += &permutation.to_string();
                    }
                    let mapped_index = match shader_map.get_mut(&shader_name) {
                        Some(n) => n,
                        None => {
                            shader_map.insert(shader_name.clone(), None);
                            shader_map.get_mut(&shader_name).unwrap()
                        }
                    };

                    // Add the shader if not present
                    if mapped_index.is_none() {
                        *mapped_index = Some(materials.len());
                        materials.push(Material {
                            name: shader_name,
                            tif_path: "<none>".to_owned()
                        });
                    }

                    // Begin extracting the geometry data
                    let shader_mapped_index = mapped_index.unwrap() as usize;

                    // Extract the vertices
                    vertices.reserve(vertices.len());
                    let first_vertex_offset = vertices.len();
                    for v in &p.uncompressed_vertices {
                        vertices.push(Vertex {
                            node0: v.node0_index,
                            node1: v.node1_index,
                            node1_weight: v.node1_weight,
                            normal: v.normal,
                            position: v.position,
                            texture_coordinates: Point3D { x: v.texture_coords.x * u_scale, y: v.texture_coords.y * v_scale, z: 1.0 }
                        });
                    }

                    // Add indices
                    let mut indices = Vec::new();
                    indices.reserve(p.triangles.len() * 3);

                    for t in &p.triangles {
                        indices.push(t.vertex0_index);
                        indices.push(t.vertex1_index);
                        indices.push(t.vertex2_index);
                    }

                    // Not enough indices to make a triangle
                    if indices.len() < 3 {
                        continue;
                    }

                    // Begin adding the triangles
                    let triangle_count = indices.len() - 2;
                    let triangle_indices = &indices[..triangle_count];
                    triangles.reserve(triangle_count);

                    let mut flipped_normal = true;
                    for t in 0..triangle_indices.len() {
                        // Get the vertex indices
                        let triangle = &indices[t..];
                        let a = triangle[0];
                        let (b, c) = match flipped_normal {
                            true => (triangle[2], triangle[1]),
                            false => (triangle[1], triangle[2])
                        };

                        // Flip the normal
                        flipped_normal = !flipped_normal;

                        // Check if degenerate or null. If so, skip
                        let a = match a { Some(n) => n as u32 , None => continue };
                        let b = match b { Some(n) => n as u32 , None => continue };
                        let c = match c { Some(n) => n as u32 , None => continue };
                        if a == b || b == c || a == c {
                            continue;
                        }

                        // Read it!
                        triangles.push ( Triangle {
                            region: Some(convert_try_into_error!(new_region_index.try_into())?),
                            shader: Some(convert_try_into_error!(shader_mapped_index.try_into())?),
                            vertices: (first_vertex_offset as u32 + a, first_vertex_offset as u32 + b, first_vertex_offset as u32 + c)
                        })
                    }
                }

                break;
            }
        }
    }

    // If we didn't get anything, return
    if triangles.is_empty() {
        return Ok(None);
    }

    Ok(Some(JMS { node_list_checksum, markers, materials, vertices, triangles, nodes, regions }))
}

pub fn recover_jms(model: Model, tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult> {
    // Let's get the path to the models dir
    let models_dir_path = match (|| {
        let mut file = data_dir.join(tag_file.tag_path.to_string());
        file.set_extension("");
        let file_name = file.file_name()?;

        let parent = file.parent()?;
        let parent_file_name = parent.file_name()?;

        if parent_file_name != file_name {
            None
        }
        else {
            Some(parent.join("models"))
        }
    })() {
        Some(n) => n,
        None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.recover.error_parent_dir_incorrect")))
    };

    // Check if the model was improperly extracted
    model.check_for_extraction_bugs()?;

    // Get all permutations
    let permutations = {
        let mut p = Vec::new();
        for region in &model.regions {
            for permutation in &region.permutations {
                let name = permutation.name.to_str();
                if !p.contains(&name) {
                    p.push(name);
                }
            }
        }
        p
    };

    // Go through each permutation
    let mut all_jms = HashMap::<&str, [Option<JMS>; 5]>::new();
    for p in permutations {
        all_jms.insert(p, [make_jms(&model, p, 0)?,
                           make_jms(&model, p, 1)?,
                           make_jms(&model, p, 2)?,
                           make_jms(&model, p, 3)?,
                           make_jms(&model, p, 4)?]);
    }

    // Now that we have all of them, let's save them!
    let mut result = RecoverResult::DataAlreadyExists;
    make_directories(&models_dir_path)?;

    for (p, jms_arr) in all_jms {
        for i in 0..jms_arr.len() {
            let jms = match jms_arr[i] {
                Some(ref n) => n,
                None => continue
            };

            let suffix = match i {
                0 => "superhigh",
                1 => "high",
                2 => "medium",
                3 => "low",
                4 => "superlow",
                _ => unreachable!()
            };

            let path = models_dir_path.join(format!("{p} {suffix}.jms"));
            if !overwrite && path.exists() {
                continue;
            }

            write_file(&path, &jms.into_bytes())?;
            result = RecoverResult::Recovered;
        }
    }

    Ok(result)
}

pub fn recover_models(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult> {
    let model = *Model::from_tag_file(tag_data)?.data;
    recover_jms(model, tag_file, data_dir, overwrite)
}

pub fn recover_gbxmodels(tag_data: &[u8], tag_file: &TagFile, data_dir: &Path, overwrite: bool) -> ErrorMessageResult<RecoverResult> {
    let model = Model::try_from(*GBXModel::from_tag_file(tag_data)?.data)?;
    recover_jms(model, tag_file, data_dir, overwrite)
}
