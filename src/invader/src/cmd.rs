//! Command-line driver for Invader.

use std::process::ExitCode;
use crate::engine;
use engine::EngineModuleFn;
use terminal::*;

use strings::get_compiled_string;

mod verb;
pub use self::verb::*;

#[macro_use]
pub mod args;

fn print_usage(path: &str, lookup: &str, engine: &dyn engine::EngineModuleFn) {
    eprintln!(get_compiled_string!("command_usage.error"), path=path);
    eprintln!();

    if !lookup.is_empty() {
        eprintln_error_pre!(get_compiled_string!("command_usage.error_no_verbs_matched"), lookup=lookup)
    }

    eprintln!(get_compiled_string!("command_usage.error_available_verbs"));

    let mut verbs_listed = 0usize;
    for v in &verb::ALL_VERBS {
        if engine.get_verb_function(v.verb).is_some() {
            verbs_listed += 1;
            eprint!("    {: <15}  {: <3}  ", v.verb.get_name(), v.verb.get_shorthand());
            let pos = 4 + 15 + 2 + 3 + 2;
            print_word_wrap(v.verb.get_description(), pos, pos, OutputType::Stderr);
        }
    }

    if verbs_listed == 0 {
        eprintln_warn!("    {}", get_compiled_string!("command_usage.error_no_verbs_available"));
    }

    eprintln!();
    eprintln!(get_compiled_string!("command_usage.error_get_help"), path=path);
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
            f(&v, &args_ref[2..], &format!("{} {}", args_ref[0], v.get_name()))
        }
        else {
            eprintln_error_pre!(get_compiled_string!("command_usage.error_verb_unsupported"), verb=v.get_name());
            ExitCode::from(2)
        }
    }
    else {
        print_usage(&args[0], &args[1], engine);
        ExitCode::from(2)
    }
}
