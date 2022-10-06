use ringhopper::engines::h1::{TagGroup, TagReference, TagFileSerializeFn, EngineTarget};
use ringhopper::engines::h1::*;
use ringhopper::engines::h1::definitions::*;
use strings::*;
use std::process::ExitCode;
use ringhopper::cmd::*;
use ringhopper::error::{ErrorMessageResult, ErrorMessage};
use ringhopper::file::*;
use ringhopper::types::*;
use crate::file::*;
use macros::terminal::*;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

struct ScriptOptions<'a> {
    reload_scripts: bool,
    regenerate: bool,
    exclude_global_scripts: bool,
    explicit: Option<&'a Vec<String>>,
    clear: bool
}

struct CompileResult {
    warning_count: usize,
    script_count: usize,
    global_count: usize
}

fn compile_scripts_for_tag(path: &TagFile,
                           data_dir: &Path,
                           hud_globals: &HUDGlobals,
                           tags_dirs: &[&Path],
                           engine_target:
                           &EngineTarget,
                           all_tags:
                           &mut Option<Vec<TagFile>>,
                           options: &ScriptOptions) -> ErrorMessageResult<CompileResult> {
    // Load the scenario tag
    let mut scenario_tag = *Scenario::from_tag_file(&read_file(&path.file_path)?)?.data;

    // Clear the source files
    let allowed_scripts = match options.reload_scripts {
        true => {
            let mut allowed_scripts = Vec::new();
            allowed_scripts.reserve(scenario_tag.source_files.blocks.len());
            for i in &scenario_tag.source_files {
                allowed_scripts.push(i.name.to_str().to_owned());
            }
            allowed_scripts.sort();
            Some(allowed_scripts)
        },
        false => None
    };

    // Get the tag data directory.
    let scenario_tag_data_dir = data_dir.join(path.tag_path.to_string()).parent().unwrap().join("scripts");

    // If we are clearing scripts, do not do anything
    if options.clear {
        scenario_tag.source_files.blocks.clear();
    }

    // If we are NOT regenerating, iterate through our tag directory.
    else if !options.regenerate {
        scenario_tag.source_files.blocks.clear();

        let global_scripts_name = "global_scripts".to_owned();
        let global_scripts_path = data_dir.join("global_scripts.hsc");

        // If explicit, add only specific scripts.
        let mut script_paths = BTreeMap::<String, PathBuf>::new();
        if let Some(n) = options.explicit {
            for script in n {
                script_paths.insert(script.to_owned(), scenario_tag_data_dir.join(format!("{script}.hsc")));
            }

            // Include global_scripts unless we exclude them from the root.
            if !options.exclude_global_scripts && n.contains(&global_scripts_name) && !script_paths.contains_key(&global_scripts_name) {
                script_paths.insert(global_scripts_name, global_scripts_path);
            }
        }

        // Otherwise, add everything and maybe the global_scripts in the root?
        else if scenario_tag_data_dir.exists() {
            // Get all the script files
            let mut all_script_files = Vec::new();
            (|| -> std::io::Result<()> {
                for file in scenario_tag_data_dir.read_dir()? {
                    let path = file?.path();

                    // If the extension is not .hsc, continue
                    match path.extension() {
                        Some(n) if n.to_ascii_lowercase() == "hsc" => (),
                        _ => continue
                    };

                    // If the path is not a file, continue.
                    if !path.is_file() {
                        continue;
                    }

                    all_script_files.push(path);
                }
                Ok(())
            })().map_err(|e| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.script.error_could_not_read_scripts"), error=e)))?;

            // Go through all scripts
            for script in all_script_files {
                // Check if ASCII
                let filename = script.file_name().unwrap();
                if !filename.is_ascii() {
                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_not_ascii"), path=script.display().to_string())));
                }

                let filename = filename.to_str().unwrap().to_owned();
                let filename_without_extension = filename.split_at(filename.len() - 4).0;

                // Check if allowed
                if allowed_scripts.is_some() && allowed_scripts.as_ref().unwrap().binary_search(&filename_without_extension.to_owned()).is_err() {
                    continue;
                }

                script_paths.insert(filename_without_extension.to_owned(), script);
            }

            // If we are including global_scripts
            if !options.exclude_global_scripts && global_scripts_path.exists() && !script_paths.contains_key(&global_scripts_name) {
                script_paths.insert(global_scripts_name, global_scripts_path);
            }
        }

        // Add them all.
        for (name,path) in script_paths {
            scenario_tag.source_files.blocks.push(ScenarioSourceFile { name: String32::from_str(&name).unwrap(), source: read_file(&path)? })
        }
    }
    else {
        // If explicit, regenerate only specific scripts.
        if let Some(n) = options.explicit {
            scenario_tag.source_files.blocks.retain(|b| n.contains(&b.name.to_str().to_owned()));
        }
    }

    // Next, load the HUD message text tags if present
    let hud_messages_tag = if !scenario_tag.hud_messages.is_empty() {
        match TagFile::from_tag_ref(&tags_dirs, &scenario_tag.hud_messages) {
            Some(n) => Ok(HUDMessageText::from_tag_file(&read_file(&n.file_path)?)?.data),
            None => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("general.error_tag_not_found"), tag=scenario_tag.hud_messages)))
        }?
    }
    else {
        Box::<HUDMessageText>::default()
    };

    // Get the warnings:
    let warnings = scenario_tag.compile_scripts(engine_target, hud_messages_tag.as_ref(), hud_globals, &mut |path| -> ErrorMessageResult<Option<TagGroup>> {
        if all_tags.is_none() {
            *all_tags = Some(TagFile::from_virtual_tags_directory(tags_dirs)?);
        }

        for i in all_tags.as_ref().unwrap() {
            if i.tag_path.get_group().is_object() && i.tag_path.get_path_without_extension() == path {
                return Ok(Some(i.tag_path.get_group()))
            }
        }

        Ok(None)
    })?;
    let warning_count = warnings.len();

    if warning_count > 0 {
        eprintln_warn!(get_compiled_string!("engine.h1.verbs.script.compiled_scripts_with_warnings"), warning_count=warning_count, tag_path=path.tag_path);
        for w in &warnings {
            let (line,column) = w.get_position();
            eprintln_warn!("    {file}:{line}:{column}: {warning}", file=w.get_file(), line=line, column=column, warning=w.get_message());
        }
    }

    // Save it
    write_file(&path.file_path, &scenario_tag.into_tag_file()?)?;

    Ok(CompileResult { warning_count, script_count: scenario_tag.scripts.len(), global_count: scenario_tag.globals.len() })
}

pub fn script_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(
        args,
        &[Argument { long: "reload-scripts", short: 'r', description: get_compiled_string!("engine.h1.verbs.script.arguments.reload_scripts.description"), parameter: None, multiple: false },
          Argument { long: "regenerate", short: 'R', description: get_compiled_string!("engine.h1.verbs.script.arguments.regenerate.description"), parameter: None, multiple: false },
          Argument { long: "exclude-global-scripts", short: 'E', description: get_compiled_string!("engine.h1.verbs.script.arguments.exclude_global_scripts.description"), parameter: None, multiple: false },
          Argument { long: "explicit", short: 'x', description: get_compiled_string!("engine.h1.verbs.script.arguments.explicit.description"), parameter: Some("source"), multiple: true },
          Argument { long: "clear", short: 'c', description: get_compiled_string!("engine.h1.verbs.script.arguments.clear.description"), parameter: None, multiple: false }],
        &[get_compiled_string!("arguments.specifier.tag_batch_without_group")],
        executable,
        verb.get_description(),
        ArgumentConstraints::new().needs_data().needs_tags().multiple_tags_directories().needs_engine()
    )?;

    // Here are our tags.
    let tags_dirs = str_slice_to_path_vec(parsed_args.named.get("tags").unwrap());

    // We first need to get the HUD globals tag from the globals tag
    let hud_globals = match (|| -> ErrorMessageResult<HUDGlobals> {
        // Get the globals tag.
        let globals_tag_reference = TagReference::from_path_and_group(r#"globals\globals"#, TagGroup::Globals).unwrap();
        let globals_tag = match TagFile::from_tag_ref(&tags_dirs, &globals_tag_reference) {
            Some(n) => Ok(Globals::from_tag_file(&read_file(&n.file_path)?)?.data),
            None => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("general.error_tag_not_found"), tag=globals_tag_reference)))
        }?;

        // Determine the file path of the HUD globals tag.
        let hud_globals_file_path = match globals_tag.interface_bitmaps.blocks.get(0) {
            None => Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.script.error_no_interface_bitmaps"))),
            Some(n) if n.hud_globals.is_empty() => Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.script.error_no_hud_globals_referenced"))),
            Some(n) => match TagFile::from_tag_ref(&tags_dirs, &n.hud_globals) {
                Some(n) => Ok(n),
                None => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("general.error_tag_not_found"), tag=n.hud_globals)))
            }
        }?;

        // Read it and return it!
        Ok(*HUDGlobals::from_tag_file(&read_file(&hud_globals_file_path.file_path)?)?.data)
    })() {
        Ok(n) => n,
        Err(e) => {
            eprintln_warn!(get_compiled_string!("engine.h1.verbs.script.error_could_not_find_hud_globals"), error=e);
            HUDGlobals::default()
        }
    };

    let tags = match TagFile::from_tag_path_batched(&str_slice_to_path_vec(&parsed_args.named["tags"]), &parsed_args.extra[0], Some(TagGroup::Scenario)) {
        Ok(n) => n,
        Err(e) => panic!("{}", e)
    };

    let options = ScriptOptions {
        reload_scripts: parsed_args.named.contains_key("reload-scripts"),
        exclude_global_scripts: parsed_args.named.contains_key("exclude-global-scripts"),
        regenerate: parsed_args.named.contains_key("regenerate"),
        explicit: parsed_args.named.get("explicit"),
        clear: parsed_args.named.contains_key("clear")
    };
    let mut processed = 0usize;
    let mut error_count = 0usize;
    let mut warning_count = 0usize;
    let mut all_tags = None;
    let data_dir = &parsed_args.named.get("data").unwrap()[0];
    let total = tags.len();
    for i in tags {
        match compile_scripts_for_tag(&i, Path::new(&data_dir), &hud_globals, &tags_dirs, parsed_args.engine_target.unwrap(), &mut all_tags, &options) {
            Ok(info) => {
                if info.script_count != 0 || info.global_count != 0 {
                    println_success!(get_compiled_string!("engine.h1.verbs.script.compiled_scripts"), tag=i.tag_path, global_count=info.script_count, script_count=info.script_count);
                }
                else {
                    println_success!(get_compiled_string!("engine.h1.verbs.script.cleared_scripts"), tag=i.tag_path);
                }
                warning_count += info.warning_count;
                processed += 1;
            },
            Err(e) => {
                eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.script.error_could_not_compile_scripts"), tag=i.tag_path, error=e);
                error_count += 1;
            }
        }
    }

    if total == 0 {
        Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_no_tags_found"))))
    }
    else if processed == 0 {
        Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.script.error_no_tags_compiled"), error=error_count)))
    }
    else if error_count > 0 {
        println_warn!(get_compiled_string!("engine.h1.verbs.script.compiled_some_tags_with_errors"), count=processed, error=error_count, warning=warning_count);
        Ok(ExitCode::FAILURE)
    }
    else {
        println_success!(get_compiled_string!("engine.h1.verbs.script.compiled_all_tags"), count=processed, warning=warning_count);
        Ok(ExitCode::SUCCESS)
    }
}
