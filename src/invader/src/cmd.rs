//! Command-line driver for Invader.

use std::process::ExitCode;
use crate::engine;
use engine::EngineModuleFn;

mod verb;
pub use self::verb::*;

fn print_usage(path: &str, lookup: &str, engine: &dyn engine::EngineModuleFn) {
    eprintln!("Usage: {path} <verb> [arguments...]");

    if lookup.is_empty() {
        eprintln!("Available verbs:");
    }
    else {
        eprintln!("No verbs matched \"{lookup}\"! Available verbs:")
    }

    let mut verbs_listed = 0usize;
    for v in &verb::ALL_VERBS {
        if engine.get_verb_function(v.verb).is_some() {
            verbs_listed += 1;
            eprintln!("    {: <15}  {: <3}  {}", v.verb.get_name(), v.verb.get_shorthand(), v.verb.get_description());
        }
    }

    if verbs_listed == 0 {
        eprintln!("    <no verbs are available for this tool>")
    }

    eprintln!("Use {path} <verb> -h to view help information for a verb.");
}

/// This is the main function for drivers to call, returning an exit code.
///
/// The exit code is 0 if successful and nonzero if a failure occurs.
pub fn main_fn<E: EngineModuleFn>(engine: &E) -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let mut args_ref: Vec<&str> = Vec::new();
    for a in &args {
        args_ref.push(a);
    }

    // No arguments?
    if args.len() == 1 {
        print_usage(args_ref[0], "", engine);
        ExitCode::from(1)
    }

    // Try to match an argument then!
    else if let Some(v) = verb::Verb::from_input(args_ref[1]) {
        if let Some(f) = engine.get_verb_function(v) {
            f(&args_ref[2..])
        }
        else {
            eprintln!("Verb \"{}\" is not supported by this tool.", v.get_name());
            ExitCode::from(2)
        }
    }
    else {
        print_usage(&args[0], &args[1], engine);
        ExitCode::from(2)
    }
}
