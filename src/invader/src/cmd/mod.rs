//! Command-line interface functionality for Invader.

use std::process::ExitCode;
use ringhopper::error::*;

mod verb;
pub use self::verb::*;

/// Execute the verb with the given arguments, returning the exit code.
pub type VerbFn = fn (verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode>;

#[macro_use]
mod args;
pub use self::args::*;
