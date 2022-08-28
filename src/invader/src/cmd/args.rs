/// Argument parsing module.

use terminal::*;
use std::collections::HashMap;
use std::path::Path;

use strings::get_compiled_string;

#[cfg(test)]
mod tests;

/// Constraints for if we need specific default resources.
#[derive(Default, Copy, Clone)]
pub struct ArgumentConstraints {
    /// True if the data dir is needed.
    pub needs_data: bool,

    /// True if the maps dir is needed.
    pub needs_maps: bool,

    /// True if the tags dirs are needed.
    pub needs_tags: bool,

    /// True if multiple tags directories are supported.
    pub multiple_tags_directories: bool
}
impl ArgumentConstraints {
    /// Instantiate a new ArgumentContraints.
    pub fn new() -> ArgumentConstraints {
        ArgumentConstraints::default()
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
    pub fn needs_maps(self) -> ArgumentConstraints {
        let mut s = self;
        s.needs_maps = true;
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
    pub extra: Vec<String>
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
const STANDARD_ARGUMENTS: [Argument; 4] = [
    Argument { long: "data", short: 'd', description: get_compiled_string!("arguments.data.description"), parameter: Some("dir"), multiple: false },
    Argument { long: "help", short: 'h', description: get_compiled_string!("arguments.help.description"), parameter: None, multiple: false },
    Argument { long: "maps", short: 'm', description: get_compiled_string!("arguments.maps.description"), parameter: Some("dir"), multiple: false },
    Argument { long: "tags", short: 't', description: get_compiled_string!("arguments.tags.description"), parameter: Some("dir"), multiple: true }
];

impl ParsedArguments {
    /// Parse the arguments from the input. The executable name/verb is put in usage_prefix.
    ///
    /// The `data`, `tags`, and `maps` arguments will always be set to a default value if they are not provided here.
    pub fn parse_arguments(input: &[&str], arguments: &[Argument], extra_arguments: &[&str], usage_prefix: &str, description: &str, constraints: ArgumentConstraints) -> Option<ParsedArguments> {
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
                        args_to_pass.push(|a| -> Option<Argument> {
                            for i in STANDARD_ARGUMENTS {
                                if i.long == a {
                                    return Some(i);
                                }
                            }
                            for i in arguments {
                                if i.long == a {
                                    return Some(i.to_owned());
                                }
                            }
                            eprintln_error_pre!(get_compiled_string!("command_usage.error_argument_unknown"), prefix="--", arg=a);
                            eprintln_error!(get_compiled_string!("arguments.error_use_help"));
                            return None
                        }(&input_param[2..])?);
                        Some(())
                    },

                    // Short parameter? (e.g. -a or -abcd)
                    Some(_) => {
                        for c in (&input_param[1..]).chars() {
                            args_to_pass.push(|a| -> Option<Argument> {
                                for i in STANDARD_ARGUMENTS {
                                    if i.short == a {
                                        return Some(i);
                                    }
                                }
                                for i in arguments {
                                    if i.short == a {
                                        return Some(i.to_owned());
                                    }
                                }
                                eprintln_error_pre!(get_compiled_string!("command_usage.error_argument_unknown"), prefix="-", arg=a);
                                eprintln_error!(get_compiled_string!("arguments.error_use_help"));
                                return None
                            }(c)?)
                        }

                        Some(())
                    }

                    // They just put "-" there?
                    None => {
                        eprintln_error_pre!(get_compiled_string!("command_usage.error_argument_unknown"), prefix="", arg=input_param);
                        eprintln_error!(get_compiled_string!("arguments.error_use_help"));
                        None
                    }
                }?;

                for a in args_to_pass {
                    // Check if we already have this
                    let v = match parsed.named.get_mut(&a.long) {
                        Some(n) => {
                            if !a.multiple {
                                eprintln_error_pre!(get_compiled_string!("command_usage.error_argument_only_usable_once"), arg=a.long);
                                eprintln_error!(get_compiled_string!("arguments.error_use_help"));
                                return None
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
                            eprintln_error_pre!(get_compiled_string!("command_usage.error_argument_takes_a_parameter"), arg=a.long);
                            eprintln_error!(get_compiled_string!("arguments.error_use_help"));
                            return None
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
                    eprintln_error_pre!(get_compiled_string!("command_usage.error_argument_unexpected"), arg=input_param);
                    eprintln_error!(get_compiled_string!("arguments.error_use_help"));
                    return None
                }

                // No? Good!
                parsed.extra.push(input_param.to_owned());
            }
        }

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
            let mut arguments_sorted = Vec::new();
            arguments_sorted.reserve(STANDARD_ARGUMENTS.len() + arguments.len());
            for i in STANDARD_ARGUMENTS {
                if i.long == "data" && !constraints.needs_data {
                    continue;
                }
                if i.long == "tags" && !constraints.needs_tags {
                    continue;
                }
                if i.long == "maps" && !constraints.needs_maps {
                    continue;
                }
                arguments_sorted.push(i);
            }
            for &i in arguments {
                arguments_sorted.push(i);
            }
            arguments_sorted.sort_by(|a, b| {
                a.short.partial_cmp(&b.short).unwrap()
            });

            let left_margin = 2;
            let argument_width = 29;
            let right_margin = 1;
            let left_side = argument_width + left_margin + right_margin;

            let terminal_width = match get_terminal_width(OutputType::Stdout) {
                n if n > left_side + 1 => n,
                _ => left_side + 1
            };

            let print_margin = |amt: usize| {
                for _ in 0..amt {
                    print!(" ");
                }
            };

            for a in arguments_sorted {
                // Print the short and long
                print_margin(left_margin);
                let mut current_position = left_margin;
                print!("-{}, --{}", a.short, a.long);
                current_position += (1 + 1 + 1) + 1 + (2 + a.long.len());

                // Any argument?
                if let Some(n) = a.parameter {
                    print_margin(1);
                    current_position += 1;
                    print!("<{}>", n);
                    current_position += 1 + n.len() + 1;
                }

                // Did we overflow?
                if current_position < (left_side - right_margin) {
                    print_margin(left_side - current_position);
                }
                else {
                    debug_assert!(false, "Left side for --{} overflowed!", a.long);
                }
                current_position = left_side;

                // Now go word through the description and print it
                let mut desc_position = 0;
                while desc_position < a.description.len() {
                    // Is it whitespace? If so, print it unless we hit the end of the line
                    if a.description.chars().nth(desc_position).unwrap() == ' ' {
                        desc_position += 1;
                        if current_position < terminal_width {
                            current_position += 1;
                            print!(" ");
                        }
                        continue;
                    }

                    // If we hit the end of the terminal, we need to newline here.
                    if current_position == terminal_width {
                        println!();
                        print_margin(left_side);
                        current_position = left_side;
                    }

                    // Get the end of the word.
                    let mut end = desc_position + 1;
                    while end < a.description.len() && a.description.chars().nth(end).unwrap() != ' ' {
                        end += 1;
                    }
                    let word_length = end - desc_position;

                    // Is the line too big for the word?
                    if current_position + word_length > terminal_width {
                        // If we are at the end of the left side, print what we have.
                        if current_position == left_side {
                            let remainder = terminal_width - current_position;
                            println!("{}", &a.description[desc_position..desc_position + remainder]);
                            desc_position += remainder;
                        }

                        // Otherwise, newline
                        else {
                            println!();
                        }

                        // Reset
                        print_margin(left_side);
                        current_position = left_side;
                    }

                    // Otherwise, print the word
                    else {
                        print!("{}", &a.description[desc_position..desc_position + word_length]);
                        current_position += word_length;
                        desc_position += word_length;
                    }
                }

                println!();
            }

            return None
        }

        // Are we missing arguments?
        if extra_arguments.len() > parsed.extra.len() {
            eprintln_error_pre!(get_compiled_string!("command_usage.error_arguments_missing"));
            for &e in extra_arguments {
                eprintln_error!(" - {e}");
            }
            eprintln_error!(get_compiled_string!("arguments.error_use_help"));
            return None
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
                    eprintln_error_pre!(get_compiled_string!("arguments.error_directory_missing"), dir=dir.display());
                    eprintln_error!(get_compiled_string!("arguments.error_use_help"));
                    return None;
                }
                if !dir.is_dir() {
                    eprintln_error_pre!(get_compiled_string!("arguments.error_directory_not_directory"), dir=dir.display());
                    eprintln_error!(get_compiled_string!("arguments.error_use_help"));
                    return None;
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
        if constraints.needs_tags {
            check_if_exists!("data");
        }

        Some(parsed)
    }
}

#[macro_export]
/// Attempt to parse the arguments, returning [None] on failure.
macro_rules! try_parse_arguments {
    ($input:expr, $arguments:expr, $extra_arguments:expr, $usage_prefix:expr, $description:expr, $constraints:expr) => {
        match ParsedArguments::parse_arguments($input, $arguments, $extra_arguments, $usage_prefix, $description, $constraints) {
            Some(n) => n,
            None => return std::process::ExitCode::from(1)
        }
    }
}

pub use try_parse_arguments;
