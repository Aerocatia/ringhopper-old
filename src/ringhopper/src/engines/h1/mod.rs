//! Halo: Combat Evolved specific functionality for Ringhopper.

mod types;
pub use self::types::*;

mod p8;
pub use self::p8::*;

mod tag_loading;
pub use self::tag_loading::*;

pub mod jms;

pub mod definitions;

mod engine_target;
pub use self::engine_target::*;

/// Maximum array length for arrays in Halo: CE.
pub const MAX_ARRAY_LENGTH: usize = i32::MAX as usize;
