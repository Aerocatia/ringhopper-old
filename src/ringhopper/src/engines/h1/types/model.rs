use crate::engines::h1::definitions::{GBXModel, Model, ModelGeometryPart, GBXModelGeometryPart};
use crate::error::*;
use strings::*;

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
