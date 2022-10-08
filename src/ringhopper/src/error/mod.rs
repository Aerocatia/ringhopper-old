//! Error-handling functions.

use std::fmt::Display;

/// A string used for error reporting.
#[derive(Clone, PartialEq, Debug)]
pub enum ErrorMessage {
    /// Statically allocated string which can be used in `const` functions.
    StaticString(&'static str),

    /// Dynamically allocated string which can be tailored to the specific error instance.
    AllocatedString(String)
}

impl Display for ErrorMessage {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::StaticString(string) => fmt.write_str(*string),
            Self::AllocatedString(string) => fmt.write_str(string)
        }
    }
}

/// Type definition of [Result] using [ErrorMessage] as the error type.
///
/// This is the main return type for error reporting from within Ringhopper's API.
pub type ErrorMessageResult<T> = Result<T, ErrorMessage>;
