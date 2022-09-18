//! Terminal-specific functions.

mod tty;
pub use self::tty::*;

/// Get the terminal width. Return [usize::MAX] if not a tty.
pub fn get_terminal_width(otype: OutputType) -> usize {
    match TTYMetadata::get_tty_metadata(otype) {
        Some(n) => n.width,
        None => usize::MAX
    }
}

/// Get whether or not colors are enabled for the given output.
pub fn get_colors_enabled(streamtype: OutputType) -> bool {
    // Check environment variable
    match std::env::var("INVADER_FORCE_COLORS") {
        Ok(n) => {
            if n == "1" {
                return true
            }
            else if n == "0" {
                return false
            }
        }
        _ => ()
    };

    // Check the TTY
    match TTYMetadata::get_tty_metadata(streamtype) {
        Some(n) => n.color,
        None => false
    }
}

/// Error mode to print with.
#[derive(Copy, Clone)]
pub enum ErrorMode {
    /// Normal color.
    Normal,

    /// Success color (light green).
    Success,

    /// Warning color (light yellow).
    Warn,

    /// Error color (light red).
    Error
}

/// Set the error color for the given stream.
#[macro_export]
macro_rules! set_error_mode_for_stream {
    ($mode:expr, $stream:expr, $streamtype:expr) => {{
        use std::io::Write;

        if get_colors_enabled($streamtype) {
            match $mode {
                ErrorMode::Normal => write!($stream, "\x1B[m"),
                ErrorMode::Success => write!($stream, "\x1B[32m"),
                ErrorMode::Warn => write!($stream, "\x1B[1;33m"),
                ErrorMode::Error => write!($stream, "\x1B[1;31m")
            }.unwrap();

            $stream.flush().unwrap();
        }
    }}
}

/// Print a line to stdout with the given [ErrorMode].
#[macro_export]
macro_rules! println_color {
    ($mode:expr, $($fmt:tt)*) => {{
        print_color!($mode, $($fmt)*);
        println!();
    }}
}

/// Print a line to stderr with the given [ErrorMode].
#[macro_export]
macro_rules! eprintln_color {
    ($mode:expr, $($fmt:tt)*) => {{
        eprint_color!($mode, $($fmt)*);
        eprintln!();
    }}
}

/// Print to stdout with the given [ErrorMode].
#[macro_export]
macro_rules! print_color {
    ($mode:expr, $($fmt:tt)*) => {{
        set_error_mode_for_stream!($mode, &mut std::io::stdout(), OutputType::Stdout);
        print!($($fmt)*);
        set_error_mode_for_stream!(ErrorMode::Normal, &mut std::io::stdout(), OutputType::Stdout);
    }}
}

/// Print to stderr with the given [ErrorMode].
#[macro_export]
macro_rules! eprint_color {
    ($mode:expr, $($fmt:tt)*) => {{
        set_error_mode_for_stream!($mode, &mut std::io::stderr(), OutputType::Stderr);
        eprint!($($fmt)*);
        set_error_mode_for_stream!(ErrorMode::Normal, &mut std::io::stderr(), OutputType::Stderr);
    }}
}

pub use eprintln_color;
pub use println_color;
pub use eprint_color;
pub use print_color;
pub use set_error_mode_for_stream;

/// Print a line to stdout with error coloring.
#[macro_export]
macro_rules! println_error {
    ($($fmt:tt)*) => {{
        println_color!(ErrorMode::Error, $($fmt)*);
    }}
}

/// Print a line to stdout with warning coloring.
#[macro_export]
macro_rules! println_warn {
    ($($fmt:tt)*) => {{
        println_color!(ErrorMode::Warn, $($fmt)*);
    }}
}

/// Print a line to stdout with success coloring.
#[macro_export]
macro_rules! println_success {
    ($($fmt:tt)*) => {{
        println_color!(ErrorMode::Success, $($fmt)*);
    }}
}

/// Print a line to stderr with error coloring.
#[macro_export]
macro_rules! eprintln_error {
    ($($fmt:tt)*) => {{
        eprintln_color!(ErrorMode::Error, $($fmt)*);
    }}
}

/// Print a line to stderr with warning coloring.
#[macro_export]
macro_rules! eprintln_warn {
    ($($fmt:tt)*) => {{
        eprintln_color!(ErrorMode::Warn, $($fmt)*);
    }}
}

/// Print to stdout with error coloring.
#[macro_export]
macro_rules! print_error {
    ($($fmt:tt)*) => {{
        print_color!(ErrorMode::Error, $($fmt)*);
    }}
}

/// Print to stdout with warning coloring.
#[macro_export]
macro_rules! print_warn {
    ($($fmt:tt)*) => {{
        print_color!(ErrorMode::Warn, $($fmt)*);
    }}
}

/// Print to stdout with success coloring.
#[macro_export]
macro_rules! print_success {
    ($($fmt:tt)*) => {{
        print_color!(ErrorMode::Success, $($fmt)*);
    }}
}

/// Print to stderr with error coloring.
#[macro_export]
macro_rules! eprint_error {
    ($($fmt:tt)*) => {{
        eprint_color!(ErrorMode::Error, $($fmt)*);
    }}
}

/// Print to stderr with warning coloring.
#[macro_export]
macro_rules! eprint_warn {
    ($($fmt:tt)*) => {{
        eprint_color!(ErrorMode::Warn, $($fmt)*);
    }}
}

/// Print a line to stderr with warning coloring prefixed with "Warning: ".
#[macro_export]
macro_rules! eprintln_warn_pre {
    ($($fmt:tt)*) => {{
        eprint_warn!(get_compiled_string!("terminal.warning_prefix"));
        eprintln_warn!($($fmt)*);
    }}
}

/// Print a line to stderr with warning coloring prefixed with "Error: ".
#[macro_export]
macro_rules! eprintln_error_pre {
    ($($fmt:tt)*) => {{
        eprint_error!(get_compiled_string!("terminal.error_prefix"));
        eprintln_error!($($fmt)*);
    }}
}

pub use eprintln_warn_pre;
pub use eprintln_error_pre;

use std::io::Write;

/// Print the text to standard output with word wrapping and an optional left column.
///
/// `current_position` is the current X position of the cursor.
pub fn print_word_wrap(string: &str, left_margin: usize, current_position: usize, otype: OutputType) {
    fn do_print<T: Write>(string: &str, left_margin: usize, current_position: usize, otype: OutputType, stream: &mut T) {
        let print_margin = |stream: &mut T, amt: usize| {
            for _ in 0..amt {
                write!(stream, " ").unwrap();
            }
        };

        // Allow for 1 extra column of leeway if needed.
        let terminal_width = match get_terminal_width(otype) {
            n if n > left_margin + 1 => n,
            _ => left_margin + 1
        };

        let mut current_position = current_position;

        // Print whitespace if we are not quite at the margin yet.
        if current_position < left_margin {
            print_margin(stream, left_margin - current_position);
            current_position = left_margin
        }

        // Now go word through the description and print it
        let mut position = 0;
        while position < string.len() {
            // Is it whitespace? If so, print it unless we hit the end of the line
            if string.chars().nth(position).unwrap() == ' ' {
                position += 1;
                if current_position < terminal_width {
                    current_position += 1;
                    write!(stream, " ").unwrap();
                }
                continue;
            }

            // If we hit the end of the terminal, we need to newline here.
            if current_position == terminal_width {
                writeln!(stream).unwrap();
                print_margin(stream, left_margin);
                current_position = left_margin;
            }

            // Get the end of the word.
            let mut end = position + 1;
            while end < string.len() && string.chars().nth(end).unwrap() != ' ' {
                end += 1;
            }
            let word_length = end - position;

            // Is the line too big for the word?
            if current_position + word_length > terminal_width {
                // If we are at the end of the left side, print what we have.
                if current_position == left_margin {
                    let remainder = terminal_width - current_position;
                    writeln!(stream, "{}", &string[position..position + remainder]).unwrap();
                    position += remainder;
                }

                // Otherwise, newline
                else {
                    writeln!(stream).unwrap();
                }

                // Reset
                print_margin(stream, left_margin);
                current_position = left_margin;
            }

            // Otherwise, print the word
            else {
                write!(stream, "{}", &string[position..position + word_length]).unwrap();
                current_position += word_length;
                position += word_length;
            }
        }
    }

    match otype {
        OutputType::Stderr => do_print(string, left_margin, current_position, otype, &mut std::io::stderr()),
        OutputType::Stdout => do_print(string, left_margin, current_position, otype, &mut std::io::stdout())
    }
}

pub use println_error;
pub use println_warn;
pub use println_success;
pub use eprintln_warn;
pub use eprintln_error;
pub use print_error;
pub use print_warn;
pub use print_success;
pub use eprint_warn;
pub use eprint_error;
