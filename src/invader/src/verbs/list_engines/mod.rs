use strings::*;
use std::process::ExitCode;
use ringhopper::cmd::*;
use ringhopper::error::ErrorMessageResult;
use ringhopper::engines::h1::ALL_TARGETS;


pub fn list_engines_verb(_verb: &Verb, _args: &[&str], _executable: &str) -> ErrorMessageResult<ExitCode> {
    let mut targets_vec = ALL_TARGETS.to_vec();
    targets_vec.retain(|t| t.shorthand.is_some());
    targets_vec.sort_by(|a,b| a.shorthand.cmp(&b.shorthand));

    println!(get_compiled_string!("engine.h1.verbs.list-engines.available_engines"));
    for target in targets_vec {
        println!("    {shorthand:20} {name}", name=target.name, shorthand=target.shorthand.unwrap_or("N/A"))
    }

    Ok(ExitCode::SUCCESS)
}
