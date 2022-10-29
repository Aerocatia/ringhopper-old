use crate::engines::h1::definitions::*;
use crate::engines::h1::*;
use crate::error::*;
use crate::types::*;

use ringhopper_proc::*;

/// Maximum number of nodes that can be addressed by a compressed vertex.
///
/// If a model has more nodes than this, then it cannot have compressed vertices.
pub const MAX_NODES_FOR_COMPRESSED_VERTICES: usize = i8::MAX as usize / 3;

/// Trait for accessing the base [`ModelGeometryPart`] of a model part.
pub trait BaseModelGeometryPart {
    /// Access the base model part for the model part.
    fn base_model_part(&self) -> &ModelGeometryPart;

    /// Access the base model part for the model part.
    fn base_model_part_mut(&mut self) -> &mut ModelGeometryPart;
}

impl BaseModelGeometryPart for GBXModelGeometryPart {
    fn base_model_part(&self) -> &ModelGeometryPart {
        &self.base_struct
    }
    fn base_model_part_mut(&mut self) -> &mut ModelGeometryPart {
        &mut self.base_struct
    }
}

impl BaseModelGeometryPart for ModelGeometryPart {
    fn base_model_part(&self) -> &ModelGeometryPart {
        self
    }
    fn base_model_part_mut(&mut self) -> &mut ModelGeometryPart {
        self
    }
}

/// Trait for repairing improperly extracted models.
pub trait ModelRepair {
    /// Check if the model was improperly extracted.
    ///
    /// This can cause issues with loading the model.
    fn check_for_extraction_bugs(&self) -> ErrorMessageResult<()>;

    /// Repair extraction bugs in the model.
    fn repair_extraction_bugs(&mut self) -> ErrorMessageResult<()>;
}

macro_rules! generate_repair_model_marker_code {
    ($t:ty) => {
        impl ModelRepair for $t {
            fn check_for_extraction_bugs(&self) -> ErrorMessageResult<()> {
                // Check if the model has markers in the wrong place.
                if !self.markers.blocks.is_empty() {
                    return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.error_improperly_extracted_model_markers")))
                }

                // Check if the model is missing uncompressed vertices.
                for g in &self.geometries {
                    for p in &g.parts {
                        let base_model_part = p.base_model_part();
                        if base_model_part.uncompressed_vertices.blocks.is_empty() && !base_model_part.compressed_vertices.blocks.is_empty() {
                            return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.error_improperly_extracted_model_vertices_uncompressed")))
                        }
                    }
                }

                Ok(())
            }

            fn repair_extraction_bugs(&mut self) -> ErrorMessageResult<()> {
                todo!()
            }
        }
    }
}

generate_repair_model_marker_code!(Model);
generate_repair_model_marker_code!(GBXModel);

macro_rules! compress_float {
    ($f:expr, $bits:expr) => {{
        const SIGNED_BIT: u32 = 1 << ($bits - 1);
        const MASK: u32 = SIGNED_BIT - 1;

        // Here is our float
        let f = $f as f32;

        // Clamp to -1 to +1
        let f = f.max(-1.0).min(1.0);

        // Compressing a float basically means taking a -1 to 1 (inclusive) value and turning it into a signed integer.
        // So, if a float is 1.0 and we need to compress it to a 16-bit integer, the result is 32767.
        // If a float is -1.0 and we need to compress it to a 16-bit integer, the result is -1 (or 32768). 0 is always 0.
        if f >= 0.0 {
            (f * (MASK as f32) + 0.5) as u32
        }
        else {
            ((1.0 + f) * (MASK as f32) + 0.5) as u32 | SIGNED_BIT
        }
    }}
}

macro_rules! decompress_float {
    ($f:expr, $bits:expr) => {{
        const SIGNED_BIT: u32 = 1 << ($bits - 1);
        const MASK: u32 = SIGNED_BIT - 1;

        // Here is our float
        let f = $f as u32;

        let number = ((f & MASK) as f32) / (MASK as f32);

        if (f & SIGNED_BIT) > 0 {
            number - 1.0
        }
        else {
            number
        }
    }}
}

fn compress_vector3(x: f32, y: f32, z: f32) -> u32 {
    (compress_float!(x, 11)) | (compress_float!(y, 11) << 11) | (compress_float!(z, 10) << 22)
}

fn decompress_vector3(v: u32) -> (f32, f32, f32) {
    (decompress_float!(v, 11), decompress_float!(v >> 11, 11), decompress_float!(v >> 22, 10))
}

/// Functions for compressing models into a smaller format.
///
/// These functions are lossy.
trait ModelCompression {
    type Compressed;

    /// Lossily compress the data.
    ///
    /// Error if it can't be done.
    fn compress(&self) -> Self::Compressed;

    /// Decompress the data.
    fn decompress(input: &Self::Compressed) -> Self;
}

impl ModelCompression for Vector3D {
    type Compressed = u32;

    fn compress(&self) -> u32 {
        compress_vector3(self.x, self.y, self.z)
    }
    fn decompress(input: &u32) -> Vector3D {
        Vector3D::from(decompress_vector3(*input))
    }
}

fn compress_node_index(node_index: Index) -> i8 {
    match node_index {
        None => -1,
        Some(n) => {
            // debug check: ensure we do not get data loss for indices; that would be VERY bad
            debug_assert!(n as usize <= MAX_NODES_FOR_COMPRESSED_VERTICES, "Vertex references node #{} which is greater than {}!", n, MAX_NODES_FOR_COMPRESSED_VERTICES);
            n as i8 * 3
        }
    }
}

fn decompress_node_index(node_index: i8) -> Index {
    match node_index {
        -1 => None,
        n => Some(n as u16 / 3)
    }
}

impl ModelCompression for ModelVertexUncompressed {
    type Compressed = ModelVertexCompressed;

    fn compress(&self) -> ModelVertexCompressed {
        ModelVertexCompressed {
            position: self.position,
            normal: self.normal.compress(),
            binormal: self.binormal.compress(),
            tangent: self.tangent.compress(),
            texture_coordinate_u: compress_float!(self.texture_coords.x, 16) as i16,
            texture_coordinate_v: compress_float!(self.texture_coords.y, 16) as i16,
            node0_index: compress_node_index(self.node0_index),
            node1_index: compress_node_index(self.node1_index),
            node0_weight: compress_float!(self.node0_weight, 16) as u16
        }
    }
    fn decompress(input: &ModelVertexCompressed) -> ModelVertexUncompressed {
        let node0_weight = decompress_float!(input.node0_weight as u32, 16);
        let node1_weight = 1.0 - node0_weight;

        ModelVertexUncompressed {
            position: input.position,
            normal: Vector3D::decompress(&input.normal),
            binormal: Vector3D::decompress(&input.binormal),
            tangent: Vector3D::decompress(&input.tangent),
            texture_coords: Point2D { x: decompress_float!(input.texture_coordinate_u, 16), y: decompress_float!(input.texture_coordinate_v, 16) },
            node0_index: decompress_node_index(input.node0_index),
            node1_index: decompress_node_index(input.node1_index),
            node0_weight,
            node1_weight
        }
    }
}

impl ModelCompression for ScenarioStructureBSPMaterialUncompressedLightmapVertex {
    type Compressed = ScenarioStructureBSPMaterialCompressedLightmapVertex;

    fn compress(&self) -> ScenarioStructureBSPMaterialCompressedLightmapVertex {
        ScenarioStructureBSPMaterialCompressedLightmapVertex {
            normal: self.normal.compress(),
            texture_coordinate_x: compress_float!(self.texture_coords.x, 16) as i16,
            texture_coordinate_y: compress_float!(self.texture_coords.y, 16) as i16,
        }
    }
    fn decompress(input: &ScenarioStructureBSPMaterialCompressedLightmapVertex) -> ScenarioStructureBSPMaterialUncompressedLightmapVertex {
        ScenarioStructureBSPMaterialUncompressedLightmapVertex {
            normal: Vector3D::decompress(&input.normal),
            texture_coords: Point2D { x: decompress_float!(input.texture_coordinate_x, 16), y: decompress_float!(input.texture_coordinate_y, 16) },
        }
    }
}

impl ModelCompression for ScenarioStructureBSPMaterialUncompressedRenderedVertex {
    type Compressed = ScenarioStructureBSPMaterialCompressedRenderedVertex;

    fn compress(&self) -> ScenarioStructureBSPMaterialCompressedRenderedVertex {
        ScenarioStructureBSPMaterialCompressedRenderedVertex {
            position: self.position,
            normal: self.normal.compress(),
            binormal: self.binormal.compress(),
            tangent: self.tangent.compress(),
            texture_coords: self.texture_coords
        }
    }
    fn decompress(input: &ScenarioStructureBSPMaterialCompressedRenderedVertex) -> ScenarioStructureBSPMaterialUncompressedRenderedVertex {
        ScenarioStructureBSPMaterialUncompressedRenderedVertex {
            position: input.position,
            normal: Vector3D::decompress(&input.normal),
            binormal: Vector3D::decompress(&input.binormal),
            tangent: Vector3D::decompress(&input.tangent),
            texture_coords: input.texture_coords
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_float_compression() {
        assert_eq!(0, compress_float!(0.0, 16));
        assert_eq!(0.0, decompress_float!(0, 16));

        assert_eq!(32768, compress_float!(-1.0, 16));
        assert_eq!(-1.0, decompress_float!(32768, 16));

        assert_eq!(32767, compress_float!(1.0, 16));
        assert_eq!(1.0, decompress_float!(32767, 16));
    }
}

/// Regenerate all compressed vertices for a model.
///
/// Return [`Err`] if compressed vertices cannot be generated. Otherwise, return `Ok(true)` if compressed vertices were generated or `Ok(false)` if compressed vertices were detected.
pub fn regenerate_compressed_vertices(model: &mut Model) -> ErrorMessageResult<bool> {
    let node_count = model.nodes.blocks.len();
    if model.nodes.blocks.len() > MAX_NODES_FOR_COMPRESSED_VERTICES {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.gbxmodel.error_cannot_regenerate_compressed_vertices_local_nodes"), node_count=node_count, limit=MAX_NODES_FOR_COMPRESSED_VERTICES)));
    }

    let mut uncompressed_vertices_exist = false;
    for g in &model.geometries {
        for p in &g.parts {
            uncompressed_vertices_exist = uncompressed_vertices_exist || !p.uncompressed_vertices.blocks.is_empty();
            if !p.compressed_vertices.blocks.is_empty() {
                return Ok(false);
            }
        }
    }

    // No uncompressed OR compressed vertices. This is a weird model. I don't want to touch it.
    if !uncompressed_vertices_exist {
        return Ok(false);
    }

    // Compress the vertices!
    for g in &mut model.geometries {
        for p in &mut g.parts {
            p.compressed_vertices.blocks.reserve_exact(p.uncompressed_vertices.blocks.len());
            for v in &p.uncompressed_vertices.blocks {
                p.compressed_vertices.blocks.push(v.compress());
            }
        }
    }

    Ok(true)
}

/// Regenerate all uncompressed vertices for a model.
///
/// Return `true` if compressed vertices were generated or `false` if compressed vertices were detected.
pub fn regenerate_uncompressed_vertices(model: &mut Model) -> bool {
    let mut compressed_vertices_exist = false;
    for g in &model.geometries {
        for p in &g.parts {
            compressed_vertices_exist = compressed_vertices_exist || !p.compressed_vertices.blocks.is_empty();
            if !p.uncompressed_vertices.blocks.is_empty() {
                return false;
            }
        }
    }

    // No uncompressed OR compressed vertices. This is a weird model. I don't want to touch it.
    if !compressed_vertices_exist {
        return false;
    }

    // Compress the vertices!
    for g in &mut model.geometries {
        for p in &mut g.parts {
            p.uncompressed_vertices.blocks.reserve_exact(p.compressed_vertices.blocks.len());
            for v in &p.compressed_vertices.blocks {
                p.uncompressed_vertices.blocks.push(ModelVertexUncompressed::decompress(v));
            }
        }
    }

    true
}
