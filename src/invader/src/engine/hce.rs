//! Halo: Combat Evolved specific functionality for Invader.

mod types;
pub use self::types::*;

mod p8;
pub use self::p8::*;

use super::Verb;
use VerbFn;
use super::EngineModuleFn;

/// Maximum array length for arrays in Halo: CE.
pub const MAX_ARRAY_LENGTH: usize = i32::MAX as usize;

/// [EngineModuleFn] interface for Halo: Combat Evolved.
#[derive(Default)]
pub struct HaloCE {}

impl EngineModuleFn for HaloCE {
    fn get_verb_function(&self, verb: Verb) -> Option<VerbFn> {
        match verb {
            _ => None
        }
    }
}
