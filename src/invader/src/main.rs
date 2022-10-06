extern crate ringhopper;
use ringhopper::engines::*;
use ringhopper::cmd::*;

extern crate strings;
extern crate macros;
extern crate flate2;
extern crate tiff;

mod verbs;
use verbs::*;

mod file;

use std::process::ExitCode;

/// [EngineModuleFn] interface for Halo: Combat Evolved.
#[derive(Default)]
pub struct HaloCE {}

impl EngineModuleFn for HaloCE {
    fn get_verb_function(&self, verb: Verb) -> Option<VerbFn> {
        match verb {
            Verb::Convert => Some(convert::convert_verb),
            Verb::ListEngines => Some(list_engines::list_engines_verb),
            Verb::Recover => Some(recover::recover_verb),
            Verb::Script => Some(script::script_verb),
            Verb::Strip => Some(strip::strip_verb),
            Verb::Strings => Some(unicode_strings::unicode_strings_verb),
            Verb::TagCollection => Some(collection::collection_verb),
            Verb::UICollection => Some(collection::collection_verb),
            Verb::UnicodeStrings => Some(unicode_strings::unicode_strings_verb),

            _ => None
        }
    }
    fn get_version(&self) -> &'static str {
        env!("invader_version")
    }
}

fn main() -> ExitCode {
    ringhopper::cmd::main_fn(&HaloCE::default())
}
