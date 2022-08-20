use std::fmt;

mod trig;
pub use self::trig::*;

mod color;
pub use self::color::*;

mod tag;
pub use self::tag::*;

use crate::{ErrorMessage, ErrorMessageResult};

/// A block of data that doesn't have any fields directly attributed to it.
pub type Data = Vec<u8>;

/// String with a maximum character length of 31 characters plus a null terminator.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct String32 {
    pub(crate) bytes: [u8; 32],
    pub(crate) length: usize
}

impl String32 {
    /// Convert into a Rust `str` reference with the same lifespan.
    pub fn to_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[0..self.length]).unwrap()
    }

    /// Convert 32 `u8` bytes into a `String32`. The input must be a valid, null terminated UTF-8 string.
    pub fn from_bytes(bytes: [u8; 32]) -> ErrorMessageResult<String32> {
        let mut length: usize = 0;

        for b in bytes {
            if b == 0 {
                break;
            }
            length += 1;
        }

        if length >= bytes.len() {
            Err(ErrorMessage::StaticString("String data is not null terminated."))
        }
        else if !std::str::from_utf8(&bytes[0..length]).is_ok() {
            Err(ErrorMessage::StaticString("String data is not valid UTF-8!"))
        }
        else {
            // Clean everything after the string
            let mut bytes_copy = bytes;
            for i in &mut bytes_copy[length..] {
                *i = 0
            }
            Ok(String32 { bytes: bytes_copy, length })
        }
    }

    /// Convert a slice of `u8` bytes into a `String32`.
    ///
    /// The input must be a valid UTF-8 string without any null termination, and it must be fewer than 32 bytes.
    pub fn from_bytes_slice(bytes: &[u8]) -> ErrorMessageResult<String32> {
        let mut input_bytes = [0u8; 32];

        // Check the length.
        let len = bytes.len();
        let limit = input_bytes.len() - 1;
        if len > limit {
            return Err(ErrorMessage::AllocatedString(format!("String data exceeds {limit} bytes. {len} > {limit}")))
        }

        // Copy!
        for b in 0..bytes.len() {
            input_bytes[b] = bytes[b];
        }

        Self::from_bytes(input_bytes)
    }

    /// Convert a string into a `String32`.
    ///
    /// The input must be a valid UTF-8 string that is fewer than 32 bytes.
    pub fn from_str(string: &str) -> ErrorMessageResult<String32> {
        Self::from_bytes_slice(string.as_bytes())
    }
}

/// Point with two X/Y integers.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Point2DInt {
    /// X value.
    pub x: i16,

    /// Y value.
    pub y: i16
}

/// Rectangle with two Point2DInts defining the bounds of the rectangle.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Rectangle {
    pub top: i16,
    pub left: i16,
    pub bottom: i16,
    pub right: i16
}

/// Tag reference that describes a tag.
///
/// For all functions that set a path, the path must be a valid path for Windows. The following characters are restricted:
///
/// - `<` (less than)
/// - `>` (greater than)
/// - `:` (colon)
/// - `"` (double quote)
/// - `/` (forward slash - will be replaced with `\`)
/// - `|` (vertical bar or pipe)
/// - `?` (question mark)
/// - `*` (asterisk)
/// - Non ASCII characters
///
/// If an invalid path is used, the string will not be written, and an [Err] will be returned with a message.
///
/// Also, all characters will be made lowercase, and the native path separator will be replaced with a backslash (`\`).
#[derive(Clone, Default)]
pub struct TagReference<T: TagGroupFn> {
    pub group: T,
    path: String
}
impl<T: TagGroupFn> TagReference<T> {
    /// Get the path without extension.
    pub fn get_path_without_extension(&self) -> &str {
        &self.path
    }

    /// Set the path without an extension.
    ///
    /// If the path is invalid for a `TagReference`, an [Err] is returned.
    pub fn set_path_without_extension(&mut self, path: &str) -> ErrorMessageResult<()> {
        let mut new_path = path.to_owned();

        // Paths must be ASCII
        if !new_path.is_ascii() {
            return Err(ErrorMessage::AllocatedString(format!("Path \"{new_path}\" is non-ASCII.")))
        }

        // Replace native path separators with Windows path separators
        if std::path::MAIN_SEPARATOR != '\\' {
            new_path = new_path.replace(std::path::MAIN_SEPARATOR, "\\");
        }

        // If the native path separator is not a forward slash, also replace forward slashes.
        if std::path::MAIN_SEPARATOR != '/' {
            new_path = new_path.replace('/', "\\");
        }

        // Check for forbidden characters
        let restricted_characters = ['<', '>', ':', '"', '/', '|', '?', '*'];
        if let Some(n) = new_path.find(&restricted_characters[..]) {
            return Err(ErrorMessage::AllocatedString(format!("Path \"{new_path}\" contains restricted character '{}'.", new_path.as_bytes()[n])))
        }

        // Make it lowercase
        new_path.make_ascii_lowercase();

        // Set it
        self.path = new_path;

        Ok(())
    }

    /// Create a `TagReference` from a path containing an extension that corresponds to a tag group.
    ///
    /// If the path is invalid for a `TagReference` or the extension is invalid or nonexistent, an [Err] is returned.
    pub fn from_path_with_extension(path: &str) -> ErrorMessageResult<TagReference<T>> {
        let pos = match path.rfind('.') {
            Some(n) => n,
            None => return Err(ErrorMessage::AllocatedString(format!("Path \"{path}\" does not have a file extension.")))
        };

        let potential_group = &path[pos+1..];
        let group = match T::from_str(potential_group) {
            Some(n) => n,
            None => return Err(ErrorMessage::AllocatedString(format!("Extension \"{potential_group}\" does not correspond to a tag group.")))
        };

        TagReference::from_path_and_group(&path[..pos], group)
    }

    /// Create a `TagReference` from a path and group.
    ///
    /// If the path is invalid for a `TagReference`, an [Err] is returned.
    pub fn from_path_and_group(path: &str, group: T) -> ErrorMessageResult<TagReference<T>> {
        let mut reference = TagReference { path: String::default(), group };
        reference.set_path_without_extension(path)?;
        Ok(reference)
    }
}

impl<T: TagGroupFn + fmt::Display> fmt::Display for TagReference<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for c in self.path.chars() {
            let mut character = c;
            if c == '\\' {
                character = std::path::MAIN_SEPARATOR;
            }
            f.write_str(std::str::from_utf8(&[character as u8]).unwrap())?;
        }
        f.write_str(".")?;

        self.group.fmt(f)
    }
}
