//! Engine specific functionality, including types and functions.

pub mod h1;
use std::process::ExitCode;
use crate::cmd::Verb;

pub mod types;

/// Execute the verb with the given arguments, returning the exit code.
pub type VerbFn = fn (verb: &Verb, args: &[&str], executable: &str) -> ExitCode;

/// Engine modules define engines supported by the driver.
#[allow(unused)]
pub trait EngineModuleFn {
    /// Get the function for the verb if it exists.
    fn get_verb_function(&self, verb: Verb) -> Option<VerbFn>;
}
