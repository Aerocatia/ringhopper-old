//! Halo: Combat Evolved specific functionality for Invader.

pub mod types;

mod p8;
pub use self::p8::*;

mod verbs;
mod unicode_string_list;
pub use self::unicode_string_list::*;

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
            Verb::UnicodeStrings => {
                Some(self::verbs::unicode_strings::unicode_strings_verb)
            }

            _ => None
        }
    }
}
