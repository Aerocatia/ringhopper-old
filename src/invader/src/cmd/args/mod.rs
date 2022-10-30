/// Argument parsing module.

use macros::terminal::*;
use std::collections::HashMap;
use std::path::Path;

use ringhopper_proc::get_compiled_string;
use ringhopper::engines::h1::EngineTarget;
use ringhopper::error::*;

use FromStr;

#[cfg(test)]
mod tests;

/// Constraints for if we need specific default resources.
#[derive(Default, Copy, Clone)]
pub struct ArgumentConstraints {
    /// True if an engine is needed.
    pub needs_engine: bool,

    /// True if the data dir is needed.
    pub needs_data: bool,

    /// True if the maps dir is needed.
    pub needs_maps: bool,

    /// True if the tags dirs are needed.
    pub needs_tags: bool,

    /// True if thread count can be specified.
    pub threads: bool,

    /// True if overwriting can be specified.
    pub overwrite: bool,

    /// True if multiple tags directories are supported.
    pub multiple_tags_directories: bool
}
impl ArgumentConstraints {
    /// Instantiate a new ArgumentContraints.
    pub fn new() -> ArgumentConstraints {
        ArgumentConstraints::default()
    }

    /// Set needs_engine to true.
    pub fn needs_engine(self) -> ArgumentConstraints {
        let mut s = self;
        s.needs_engine = true;
        s
    }

    /// Set needs_data to true.
    pub fn needs_data(self) -> ArgumentConstraints {
        let mut s = self;
        s.needs_data = true;
        s
    }

    /// Set needs_tags to true.
    pub fn needs_tags(self) -> ArgumentConstraints {
        let mut s = self;
        s.needs_tags = true;
        s
    }

    /// Set needs_maps to true.
    #[allow(unused)]
    pub fn needs_maps(self) -> ArgumentConstraints {
        let mut s = self;
        s.needs_maps = true;
        s
    }

    /// Set overwrite to true.
    pub fn can_overwrite(self) -> ArgumentConstraints {
        let mut s = self;
        s.overwrite = true;
        s
    }

    /// Set overwrite to true.
    pub fn uses_threads(self) -> ArgumentConstraints {
        let mut s = self;
        s.threads = true;
        s
    }

    /// Set multiple_tags_directories to true.
    pub fn multiple_tags_directories(self) -> ArgumentConstraints {
        let mut s = self;
        s.multiple_tags_directories = true;
        s
    }
}


/// Argument to search for in [ParsedArguments::parse_arguments].
#[derive(Default)]
pub struct ParsedArguments {
    /// Named arguments
    pub named: HashMap<&'static str, Vec<String>>,

    /// Extra arguments that do not have a switch associated with them
    pub extra: Vec<String>,

    /// Engine target, if one was specified
    pub engine_target: Option<&'static EngineTarget>,

    /// Number of threads to use
    pub threads: usize
}

impl ParsedArguments {
    /// Get the first argument, parsed as a 32-bit float if set.
    pub fn parse_f32(&self, argument: &str) -> ErrorMessageResult<Option<f32>> {
        match self.named.get(argument) {
            Some(n) => {
                let parsed_float = n[0].parse::<f32>();
                match parsed_float {
                    Ok(f) => Ok(Some(f)),
                    Err(e) => Err(ErrorMessage::AllocatedString(e.to_string()))
                }
            },
            None => Ok(None)
        }
    }

    /// Get the first argument, parsed as a 16-bit unsigned integer if set.
    pub fn parse_u16(&self, argument: &str) -> ErrorMessageResult<Option<u16>> {
        match self.named.get(argument) {
            Some(n) => {
                let parsed_u16 = n[0].parse::<u16>();
                match parsed_u16 {
                    Ok(i) => Ok(Some(i)),
                    Err(e) => Err(ErrorMessage::AllocatedString(e.to_string()))
                }
            },
            None => Ok(None)
        }
    }

    /// Get the first argument, parsed as an enum value if set.
    pub fn parse_enum<T: FromStr<Err = ErrorMessage>>(&self, argument: &str) -> ErrorMessageResult<Option<T>> {
        match self.named.get(argument) {
            Some(n) => Ok(Some(crate::from_str(&n[0])?)),
            None => Ok(None)
        }
    }

    /// Get the first argument, parsed as some value if set.
    pub fn parse_set<T>(&self, argument: &str, allowed_values: &[(&str, T)]) -> ErrorMessageResult<Option<T>> where T: Sized + Copy {
        let arg = match self.named.get(argument) {
            Some(n) => &n[0],
            None => return Ok(None)
        };

        for i in allowed_values {
            if arg == i.0 {
                return Ok(Some(i.1));
            }
        }

        let mut error_message = format!("{arg} != \"{first_val}\"", first_val=allowed_values[0].0);
        for i in &allowed_values[1..] {
            error_message += &format!(" | \"{value}\"", value=i.0);
        }
        Err(ErrorMessage::AllocatedString(error_message))
    }

    // Parse a boolean on/off value.
    pub fn parse_bool_on_off(&self, argument: &str) -> ErrorMessageResult<Option<bool>> {
        self.parse_set(argument, &[("on", true), ("off", false)])
    }
}


/// Argument to search for in [ParsedArguments::parse_arguments].
#[derive(Copy, Clone)]
pub struct Argument {
    /// Long name for the argument.
    pub long: &'static str,

    /// Short symbol for the argument.
    pub short: char,

    /// Description for the argument.
    pub description: &'static str,

    /// Parameter name if the argument takes a parameter.
    pub parameter: Option<&'static str>,

    /// True if the argument can be used multiple times.
    pub multiple: bool
}

/// Arguments present in all commands
const STANDARD_ARGUMENTS: &'static [Argument] = &[
    Argument { long: "data", short: 'd', description: get_compiled_string!("arguments.data.description"), parameter: Some("dir"), multiple: false },
    Argument { long: "help", short: 'h', description: get_compiled_string!("arguments.help.description"), parameter: None, multiple: false },
    Argument { long: "maps", short: 'm', description: get_compiled_string!("arguments.maps.description"), parameter: Some("dir"), multiple: false },
    Argument { long: "tags", short: 't', description: get_compiled_string!("arguments.tags.description_multi"), parameter: Some("dir"), multiple: true },
    Argument { long: "tags", short: 't', description: get_compiled_string!("arguments.tags.description_single"), parameter: Some("dir"), multiple: false },
    Argument { long: "engine", short: 'e', description: get_compiled_string!("arguments.engine.description"), parameter: Some("engine"), multiple: false },

    Argument { long: "threads", short: 'j', description: get_compiled_string!("arguments.threads.description"), parameter: Some("threads"), multiple: false },
    Argument { long: "overwrite", short: 'o', description: get_compiled_string!("arguments.overwrite.description"), parameter: None, multiple: false }
];

impl ParsedArguments {
    /// Parse the arguments from the input. The executable name/verb is put in usage_prefix.
    ///t
    /// The `data`, `tags`, and `maps` arguments will always be set to a default value if they are not provided here.
    pub fn parse_arguments(input: &[&str], arguments: &[Argument], extra_arguments: &[&str], usage_prefix: &str, description: &str, constraints: ArgumentConstraints) -> ErrorMessageResult<ParsedArguments> {
        (|| {
            debug_assert!({
                let argument_count = arguments.len();
                for i in 0..argument_count {
                    for j in 0..i {
                        assert_ne!(arguments[j].long, arguments[i].long, "{} conflicts with another argument {}'s long parameter", arguments[i].long, arguments[j].long);
                        assert_ne!(arguments[j].short, arguments[i].short, "{} conflicts with another argument {}'s long parameter", arguments[i].long, arguments[j].long);
                    }
                    for s in STANDARD_ARGUMENTS {
                        assert_ne!(s.long, arguments[i].long, "{} conflicts with standard argument {}'s long parameter", arguments[i].long, s.long);
                        assert_ne!(s.short, arguments[i].short, "{} conflicts with standard argument {}'s short parameter", arguments[i].long, s.long);
                    }
                    assert!(!(arguments[i].parameter.is_none() && arguments[i].multiple), "{} takes no parameter but can be used multiple times", arguments[i].long);
                }
                true
            });

            // Concatenate all arguments
            let mut available_arguments = STANDARD_ARGUMENTS.to_vec();
            available_arguments.retain(|n| {
                !(n.short == 'o' && !constraints.overwrite) &&
                !(n.short == 'j' && !constraints.threads)

                // note that we allow --data, --maps, and --tags in all verbs as these define important directories and might be passed in scripts, etc.,
                // thus we can just ignore them if they aren't used
            });
            available_arguments.extend_from_slice(arguments);

            let mut parsed = ParsedArguments::default();
            parsed.extra.reserve_exact(extra_arguments.len());

            let mut input_index = 0usize;
            while input_index < input.len() {
                let input_param = input[input_index];
                input_index += 1;

                // Parameter?
                if let Some('-') = input_param.chars().nth(0) {
                    let mut args_to_pass: Vec<Argument> = Vec::new();

                    match input_param.chars().nth(1) {
                        // Long parameter? (e.g. --help or --tags)
                        Some('-') => {
                            args_to_pass.push(|a| -> ErrorMessageResult<Argument> {
                                for i in &available_arguments {
                                    if i.long == a {
                                        return Ok(i.to_owned());
                                    }
                                }
                                Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("command_usage.error_argument_unknown"), prefix="--", arg=a)))
                            }(&input_param[2..])?);
                            Ok(())
                        },

                        // Short parameter? (e.g. -a or -abcd)
                        Some(_) => {
                            for c in (&input_param[1..]).chars() {
                                args_to_pass.push(|a| -> ErrorMessageResult<Argument> {
                                    for i in &available_arguments {
                                        if i.short == a {
                                            return Ok(i.to_owned());
                                        }
                                    }
                                Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("command_usage.error_argument_unknown"), prefix="-", arg=a)))
                                }(c)?)
                            }

                            Ok(())
                        }

                        // They just put "-" there?
                        None => {
                            Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("command_usage.error_argument_unknown"), prefix="", arg=input_param)))
                        }
                    }?;

                    for a in args_to_pass {
                        // Check if we already have this
                        let v = match parsed.named.get_mut(&a.long) {
                            Some(n) => {
                                if !a.multiple {
                                    return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("command_usage.error_argument_only_usable_once"), arg=a.long)));
                                }
                                n
                            },
                            None => {
                                parsed.named.insert(a.long, Vec::new());
                                parsed.named.get_mut(&a.long).unwrap()
                            }
                        };

                        // We take a parameter but we don't have any left!
                        if a.parameter.is_some() {
                            if input_index == input.len() {
                                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("command_usage.error_argument_takes_a_parameter"), arg=a.long)));
                            }
                            v.push(input[input_index].to_owned());
                            input_index += 1;
                            continue;
                        }
                    }
                }

                // OK
                else {
                    // Did we hit the end?
                    if parsed.extra.len() == extra_arguments.len() {
                        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("command_usage.error_argument_unexpected"), arg=input_param)));
                    }

                    // No? Good!
                    parsed.extra.push(input_param.to_owned());
                }
            }

            parsed.threads = if let Some(t) = parsed.named.get("threads") {
                let string = t[0].as_str();
                string.parse().map_err(|e| ErrorMessage::AllocatedString(format!(get_compiled_string!("command_usage.error_bad_thread_count"), string=string, error=e)))?
            } else {
                match std::thread::available_parallelism() {
                    Ok(n) => n.get(),
                    Err(_) => 1
                }
            };

            // Did we put --help? If so... yay.
            if let Some(_) = parsed.named.get("help") {
                print!(
                    "Usage: {usage_prefix} [options]{extra_arguments}\n\n{description}\n\nOptions:\n",
                    extra_arguments={
                        match extra_arguments.len() {
                            0 => String::new(),
                            1 => format!(" <{}>", extra_arguments[0]),
                            _ => {
                                let mut a = String::new();

                                for e in extra_arguments {
                                    a += &format!(" <{}>", e);
                                }

                                a
                            }
                        }
                    }
                );

                // Get all arguments and sort them.
                available_arguments.sort_by(|a, b| {
                    a.short.partial_cmp(&b.short).unwrap()
                });

                // Hide unused parameters
                available_arguments.retain(|n| {
                    !(n.short == 't' && (
                        !constraints.needs_tags ||
                        (n.description == get_compiled_string!("arguments.tags.description_multi") && !constraints.multiple_tags_directories) ||
                        (n.description == get_compiled_string!("arguments.tags.description_single") && constraints.multiple_tags_directories)
                    )) &&
                    !(n.short == 'd' && !constraints.needs_data) &&
                    !(n.short == 'm' && !constraints.needs_maps) &&
                    !(n.short == 'e' && !constraints.needs_engine)
                });

                let argument_width = 41;
                let left_side = argument_width;

                for a in available_arguments {
                    // Print the short and long
                    print!("  -{}, --{}", a.short, a.long);
                    let mut current_position = 2 + (1 + 1 + 1) + 1 + (2 + a.long.len());

                    // Any argument?
                    if let Some(n) = a.parameter {
                        print!(" <{}>", n);
                        current_position += 1 + (1 + n.len() + 1);
                    }

                    // Did we overflow?
                    debug_assert!(current_position < left_side, "Left side for --{} overflowed!", a.long);

                    // Print it!
                    print_word_wrap(a.description, left_side, current_position, OutputType::Stdout);

                    // Done
                    println!();
                }

                return Err(ErrorMessage::StaticString(""));
            }

            // Are we missing arguments?
            if extra_arguments.len() > parsed.extra.len() {
                let mut missing_things = get_compiled_string!("command_usage.error_arguments_missing").to_owned();
                for &e in extra_arguments {
                    missing_things += &format!("\n - {e}");
                }
                return Err(ErrorMessage::AllocatedString(missing_things));
            }

            macro_rules! check_if_exists {
                ($key:expr) => {
                    // Check if we supplied the directory. If not, add a default.
                    if !parsed.named.contains_key($key) {
                        parsed.named.insert($key, vec![$key.to_owned()]);
                    }

                    // Now let's check if it exists.
                    let dir: &Path = Path::new(&parsed.named.get($key).unwrap()[0]);
                    if !dir.exists() {
                        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("arguments.error_directory_missing"), dir=dir.display())));
                    }
                    if !dir.is_dir() {
                        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("arguments.error_directory_not_directory"), dir=dir.display())));
                    }
                }
            }

            // Verify everything exists.
            if constraints.needs_maps {
                check_if_exists!("maps");
            }
            if constraints.needs_tags {
                check_if_exists!("tags");

                if !constraints.multiple_tags_directories {
                    let dm = parsed.named.get_mut("tags").unwrap();
                    if dm.len() > 1 {
                        dm.resize(1, String::new());
                        eprintln_warn_pre!(get_compiled_string!("command_usage.warning_only_one_tags_dir_supported"), dir=&dm[0]);
                    }
                }
            }
            if constraints.needs_data {
                check_if_exists!("data");
            }
            if constraints.needs_engine {
                parsed.engine_target = match parsed.named.get("engine") {
                    None => return Err(ErrorMessage::StaticString(get_compiled_string!("arguments.error_engine_needed"))),
                    Some(n) => match EngineTarget::from_shorthand(&n[0]) {
                        Some(n) => Some(n),
                        None => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("arguments.error_engine_invalid"), engine=n[0])))
                    }
                };
            }

            Ok(parsed)
        })().map_err(|e| {
            if e == ErrorMessage::StaticString("") { // empty error message (used with --help so we don't print extra text)
                e
            }
            else {
                ErrorMessage::AllocatedString(format!("{e}\n{help}", help=get_compiled_string!("arguments.error_use_help")))
            }
        })
    }
}
