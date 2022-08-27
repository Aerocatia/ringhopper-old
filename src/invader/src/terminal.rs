/// Get the terminal width. Return [usize::MAX] if no tty.
pub fn get_terminal_width() -> usize {
    match termion::terminal_size() {
        Ok((w, _)) if w > 1 => w as usize,
        _ => usize::MAX
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
    ($mode:expr, $stream:expr) => {{
        use std::io::Write;

        if termion::is_tty($stream) {
            match $mode {
                ErrorMode::Normal => write!($stream, "{}", termion::color::Fg(termion::color::Reset)),
                ErrorMode::Success => write!($stream, "{}", termion::color::Fg(termion::color::LightGreen)),
                ErrorMode::Warn => write!($stream, "{}", termion::color::Fg(termion::color::LightYellow)),
                ErrorMode::Error => write!($stream, "{}", termion::color::Fg(termion::color::LightRed))
            }.unwrap();

            $stream.flush().unwrap();
        }
    }}
}

/// Print a line to stdout with the given [ErrorMode].
#[macro_export]
macro_rules! println_color {
    ($mode:expr, $($fmt:tt)*) => {{
        set_error_mode_for_stream!($mode, &mut std::io::stdout());
        println!($($fmt)*);
        set_error_mode_for_stream!(ErrorMode::Normal, &mut std::io::stdout());
    }}
}

/// Print a line to stderr with the given [ErrorMode].
#[macro_export]
macro_rules! eprintln_color {
    ($mode:expr, $($fmt:tt)*) => {{
        set_error_mode_for_stream!($mode, &mut std::io::stderr());
        eprintln!($($fmt)*);
        set_error_mode_for_stream!(ErrorMode::Normal, &mut std::io::stderr());
    }}
}

/// Print to stdout with the given [ErrorMode].
#[macro_export]
macro_rules! print_color {
    ($mode:expr, $($fmt:tt)*) => {{
        set_error_mode_for_stream!($mode, &mut std::io::stdout());
        print!($($fmt)*);
        set_error_mode_for_stream!(ErrorMode::Normal, &mut std::io::stdout());
    }}
}

/// Print to stderr with the given [ErrorMode].
#[macro_export]
macro_rules! eprint_color {
    ($mode:expr, $($fmt:tt)*) => {{
        set_error_mode_for_stream!($mode, &mut std::io::stderr());
        eprint!($($fmt)*);
        set_error_mode_for_stream!(ErrorMode::Normal, &mut std::io::stderr());
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

/// Print a line to stderr with warning coloring prefixed with "Warning: ".
#[macro_export]
macro_rules! eprintln_warn_pre {
    ($($fmt:tt)*) => {{
        eprint_error!(get_compiled_string!("terminal.warning_prefix"));
        eprintln_error!($($fmt)*);
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
