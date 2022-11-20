use ringhopper::engines::h1::{TagGroup, TagReference, TagFileSerializeFn, EngineTarget};
use ringhopper::engines::h1::*;
use ringhopper::engines::h1::definitions::*;
use ringhopper_proc::*;
use std::num::NonZeroUsize;
use std::process::ExitCode;
use crate::cmd::*;
use ringhopper::error::{ErrorMessageResult, ErrorMessage};
use ringhopper::file::*;
use ringhopper::types::*;
use crate::file::*;
use macros::terminal::*;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct ScriptOptions {
    tags_dirs: Vec<PathBuf>,
    all_tags: Arc<Mutex<Option<Vec<TagFile>>>>,
    hud_globals: HUDGlobals,
    data_dir: PathBuf,
    engine_target: &'static EngineTarget,
    reload_scripts: bool,
    regenerate: bool,
    exclude_global_scripts: bool,
    explicit: Option<Vec<String>>,
    clear: bool
}

fn compile_scripts_for_tag(path: &TagFile, log_mutex: super::LogMutex, _available_threads: NonZeroUsize, options: &ScriptOptions) -> ErrorMessageResult<bool> {
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
    let scenario_tag_data_dir = options.data_dir.join(path.tag_path.to_string()).parent().unwrap().join("scripts");

    // If we are clearing scripts, do not do anything
    if options.clear {
        scenario_tag.source_files.blocks.clear();
    }

    // If we are NOT regenerating, iterate through our tag directory.
    else if !options.regenerate {
        scenario_tag.source_files.blocks.clear();

        let global_scripts_name = "global_scripts".to_owned();
        let global_scripts_path = options.data_dir.join("global_scripts.hsc");

        // If explicit, add only specific scripts.
        let mut script_paths = BTreeMap::<String, PathBuf>::new();
        if let Some(n) = &options.explicit {
            for script in n {
                script_paths.insert(script.to_owned(), scenario_tag_data_dir.join(format!("{script}.hsc")));
            }

            // Include global_scripts unless we exclude them from the root.
            if !options.exclude_global_scripts && n.contains(&global_scripts_name) && !script_paths.contains_key(&global_scripts_name) {
                script_paths.insert(global_scripts_name.clone(), global_scripts_path.clone());
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
                script_paths.insert(global_scripts_name.clone(), global_scripts_path.clone());
            }
        }

        // Add them all.
        for (name,path) in &script_paths {
            scenario_tag.source_files.blocks.push(ScenarioSourceFile { name: String32::from_str(&name).unwrap(), source: read_file(&path)? })
        }

        // Sort them.
        scenario_tag.source_files.blocks.sort_by(|a, b| a.name.to_str().cmp(b.name.to_str()));

        // Check if we have a global_scripts that isn't from the root.
        //
        // This is not allowed because recovery would be lossy, as it wouldn't be recovered to the scripts dir but the root dir instead.
        match script_paths.get(&global_scripts_name) {
            Some(n) if n != &global_scripts_path => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.verbs.script.error_global_scripts_not_in_root"))),
            Some(_) => {
                // Move global_scripts to the top
                let global_scripts_index = scenario_tag.source_files.blocks.binary_search_by(|f| f.name.to_str().cmp("global_scripts")).unwrap();
                let global_scripts = scenario_tag.source_files.blocks.remove(global_scripts_index);
                scenario_tag.source_files.blocks.insert(0, global_scripts);
            }
            _ => ()
        }
    }
    else {
        // If explicit, regenerate only specific scripts.
        if let Some(n) = &options.explicit {
            scenario_tag.source_files.blocks.retain(|b| n.contains(&b.name.to_str().to_owned()));
        }
    }

    // Next, load the HUD message text tags if present
    let hud_messages_tag = if !scenario_tag.hud_messages.is_empty() {
        let tags_dirs: Vec<&Path> = options.tags_dirs.iter().map(|f| f.as_path()).collect();

        match TagFile::from_tag_ref(&tags_dirs, &scenario_tag.hud_messages) {
            Some(n) => Ok(HUDMessageText::from_tag_file(&read_file(&n.file_path)?)?.data),
            None => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("general.error_tag_not_found"), tag=scenario_tag.hud_messages)))
        }?
    }
    else {
        Box::<HUDMessageText>::default()
    };

    // Get the warnings:
    let warnings = scenario_tag.compile_scripts(options.engine_target, hud_messages_tag.as_ref(), &options.hud_globals, &mut |path| -> ErrorMessageResult<Option<TagGroup>> {
        let mut all_tags = options.all_tags.lock().unwrap();
        if all_tags.is_none() {
            let tags_dirs: Vec<&Path> = options.tags_dirs.iter().map(|f| f.as_path()).collect();
            *all_tags = Some(TagFile::from_virtual_tags_directory(&tags_dirs)?);
        }
        for i in all_tags.as_ref().unwrap() {
            if i.tag_path.get_group().is_object() && i.tag_path.get_path_without_extension() == path {
                return Ok(Some(i.tag_path.get_group()))
            }
        }
        Ok(None)
    })?;
    let warning_count = warnings.len();
    let script_count = scenario_tag.scripts.len();
    let global_count = scenario_tag.globals.len();

    // Save it
    write_file(&path.file_path, &scenario_tag.into_tag_file()?)?;

    let l = log_mutex.lock();

    if warning_count > 0 {
        eprintln_warn!(get_compiled_string!("engine.h1.verbs.script.compiled_scripts_with_warnings"), warning_count=warning_count, tag_path=path.tag_path);
        for w in &warnings {
            let (line,column) = w.get_position();
            eprintln_warn!("    {file}:{line}:{column}: {warning}", file=w.get_file(), line=line, column=column, warning=w.get_message());
        }
    }
    else if script_count != 0 || global_count != 0 {
        println_success!(get_compiled_string!("engine.h1.verbs.script.compiled_scripts"), tag=path.tag_path, global_count=global_count, script_count=script_count);
    }
    else {
        println_success!(get_compiled_string!("engine.h1.verbs.script.cleared_scripts"), tag=path.tag_path);
    }

    drop(l);

    Ok(true)
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
        ArgumentConstraints::new().needs_data().needs_tags().multiple_tags_directories().needs_engine().uses_threads()
    )?;

    // Here are our tags.
    let tags_dirs = str_slice_to_path_vec(&parsed_args.named["tags"]);

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

    let options = ScriptOptions {
        hud_globals,
        all_tags: Arc::new(Mutex::new(None)),
        tags_dirs: parsed_args.named["tags"].iter().map(|p| Path::new(p).to_owned()).collect(),
        data_dir: Path::new(&parsed_args.named.get("data").unwrap()[0]).to_owned(),
        engine_target: parsed_args.engine_target.unwrap(),
        reload_scripts: parsed_args.named.contains_key("reload-scripts"),
        exclude_global_scripts: parsed_args.named.contains_key("exclude-global-scripts"),
        regenerate: parsed_args.named.contains_key("regenerate"),
        explicit: parsed_args.named.get("explicit").map(|f| f.to_owned()),
        clear: parsed_args.named.contains_key("clear")
    };

    let tag_path = &parsed_args.extra[0];
    Ok(super::do_with_batching_threaded(compile_scripts_for_tag, tag_path, Some(TagGroup::Scenario), &str_slice_to_path_vec(&parsed_args.named["tags"]), parsed_args.threads, options)?.exit_code())
}
