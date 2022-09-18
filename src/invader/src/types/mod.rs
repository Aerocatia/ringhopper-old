//! Types of fields used in tag structs.

use std::cmp::PartialEq;
use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use std::fmt;

pub mod tag;
use self::tag::*;

mod trig;
pub use self::trig::*;

mod color;
pub use self::color::*;

use crate::error::*;

use strings::get_compiled_string;

/// FourCC type (aliased to a 32-bit unsigned integer)
pub type FourCC = u32;

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
            Err(ErrorMessage::StaticString(get_compiled_string!("engine.types.error_string_not_null_terminated")))
        }
        else if !std::str::from_utf8(&bytes[0..length]).is_ok() {
            Err(ErrorMessage::StaticString(get_compiled_string!("engine.types.error_string_not_valid_utf8")))
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
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_string_exceeds_limit"), limit=limit, len=len)))
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
    /// Position of the top edge of the rectangle.
    pub top: i16,

    /// Position of the left edge of the rectangle.
    pub left: i16,

    /// Position of the bottom edge of the rectangle.
    pub bottom: i16,

    /// Position of the right edge of the rectangle.
    pub right: i16
}

/// Value composed of both an upper and lower bound value.
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct Bounds<T> {
    /// Lower bound value.
    pub lower: T,

    /// Upper bound value.
    pub upper: T
}

impl<T: PartialOrd + Copy> Bounds<T> {
    /// Check if the upper bound is greater than or equal to the lower bound.
    pub fn is_normal(&self) -> bool {
        self.upper >= self.lower
    }

    /// Return a bounds value, setting the lower bound to the upper bound if lower is greater than upper.
    ///
    /// Otherwise, return the value unmodified.
    pub fn normalize_lower(self) -> Bounds<T> {
        if self.lower > self.upper {
            Bounds { lower: self.upper, upper: self.upper }
        }
        else {
            self
        }
    }

    /// Return a bounds value, setting the upper bound to the lower bound if upper is less than lower.
    ///
    /// Otherwise, return the value unmodified.
    pub fn normalize_upper(self) -> Bounds<T> {
        if self.upper < self.lower {
            Bounds { lower: self.lower, upper: self.lower }
        }
        else {
            self
        }
    }
}

fn match_pattern_bytes(string: &[u8], pattern: &[u8]) -> bool {
    let string_len = string.len();
    let pattern_len = pattern.len();

    let mut string_index = 0usize;
    let mut pattern_index = 0usize;

    while string_index < string_len && pattern_index < pattern_len {
        let p = pattern[pattern_index] as char;
        let s = string[string_index] as char;

        // Match single character (? matches anything, / matches path separators)
        if (p == s && p != '*') || p == '?' || ((p == '/' || p == '\\' || p == std::path::MAIN_SEPARATOR) && (s == '/' || s == '\\' || s == std::path::MAIN_SEPARATOR)) {
            pattern_index += 1;
            string_index += 1;
        }
        else if p == '*' {
            // Skip this wildcard and all consecutive wildcards
            while pattern_index < pattern_len && pattern[pattern_index] as char == '*' {
                pattern_index += 1;
            }

            // Now try to match the end of the string
            for i in string_index..=string_len {
                if match_pattern_bytes(&string[i..], &pattern[pattern_index..]) {
                    return true
                }
            }

            // Nope!
            return false;
        }
        else {
            // Mismatch!
            return false;
        }
    }

    // If both hit the end, yay!
    string_index == string_len && pattern_index == pattern_len
}

/// Match the tag path pattern.
///
/// This returns true if pattern and string are equal under the following rules:
/// - `*` refers to a wildcard that can be any number of characters.
/// - `?` refers to a wildcard that can be any one character.
/// - `/`, `\`, and the native path separators are all considered the same character.
///
/// Note that wildcard characters cannot be directly matched. These characters are unsupported in tag paths, anyway.
pub fn match_pattern(string: &str, pattern: &str) -> bool {
    match_pattern_bytes(string.as_bytes(), pattern.as_bytes())
}

/// Tag reference that describes a tag.
///
/// For all functions that set a path, the path must be a valid path for Windows. The following characters are
/// restricted:
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
///
/// Note that the [`to_string`](std::string::ToString::to_string) function and [`Display`](std::fmt::Display) will
/// display native path separators. Use [`get_path_with_extension`](TagReference::get_path_with_extension) if you want
/// to get a non-OS dependent path.
#[derive(Clone, Default, PartialEq)]
pub struct TagReference<T: TagGroupFn> {
    pub group: T,
    path: String
}
impl<T: TagGroupFn> TagReference<T> {
    /// Get the path without the file extension.
    ///
    /// This will return the path with Halo separators (i.e. backslashes).
    pub fn get_path_without_extension(&self) -> &str {
        &self.path
    }

    /// Get the path concatenated with the file extension.
    ///
    /// This will return the path with Halo separators (i.e. backslashes).
    pub fn get_path_with_extension(&self) -> String {
        format!("{}.{}", self.path, self.group.as_str())
    }

    /// Convert to a relative filesystem path.
    ///
    /// This is a convenience function for using [`to_string`](std::string::ToString::to_string) and wrapping the
    /// result in a [`PathBuf`].
    pub fn get_relative_fs_path(&self) -> PathBuf {
        PathBuf::from(self.to_string())
    }

    /// Set the path without an extension.
    ///
    /// Native path separators are converted into Halo path separators (i.e. backslashes).
    ///
    /// If the path is invalid for a `TagReference`, an [`Err`] is returned.
    pub fn set_path_without_extension(&mut self, path: &str) -> ErrorMessageResult<()> {
        let mut new_path = path.as_bytes().to_owned();

        for c in &mut new_path {
            let character = *c as char;

            // Allow any lowercase characters or numeric characters
            if character.is_ascii_lowercase() || character.is_ascii_digit() {
                continue;
            }

            // Make uppercase characters lowercase
            else if character.is_ascii_uppercase() {
                *c = c.to_ascii_lowercase();
            }

            // Convert path separators
            else if character == std::path::MAIN_SEPARATOR || character == '\\' || character == '/' {
                *c = '\\' as u8;
            }

            // Ban non-ascii
            else if !character.is_ascii() {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_not_ascii"), new_path=path)))
            }

            // Ban these characters
            else if ['<', '>', ':', '"', '/', '|', '?', '*'].contains(&character) {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_has_restricted_character"), new_path=path, character=character)))
            }
        }

        // Set it
        self.path = String::from_utf8(new_path).unwrap();

        Ok(())
    }

    /// Create a `TagReference` from a path containing an extension that corresponds to a tag group.
    ///
    /// If the path is invalid for a `TagReference` or the extension is invalid or nonexistent, an [`Err`] is returned.
    pub fn from_path_with_extension(path: &str) -> ErrorMessageResult<TagReference<T>> {
        let pos = match path.rfind('.') {
            Some(n) => n,
            None => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_missing_extension"), path=path)))
        };

        let (path_without_group, extension) = path.split_at(pos);
        let group = match T::from_str(&extension[1..]) {
            Some(n) => n,
            None => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_group_invalid"), potential_group=extension)))
        };

        TagReference::from_path_and_group(path_without_group, group)
    }

    /// Create a `TagReference` from a path and group.
    ///
    /// If the path is invalid for a `TagReference`, an [`Err`] is returned.
    pub fn from_path_and_group(path: &str, group: T) -> ErrorMessageResult<TagReference<T>> {
        let mut reference = TagReference { path: String::default(), group };
        reference.set_path_without_extension(path)?;
        Ok(reference)
    }

    /// Check if the path matches the pattern.
    ///
    /// See [`match_pattern`] for more information.
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        match_pattern(&self.get_path_with_extension(), pattern)
    }
}

impl<T: TagGroupFn> fmt::Display for TagReference<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for c in self.path.chars() {
            let mut character = c;
            if c == '\\' {
                character = std::path::MAIN_SEPARATOR;
            }
            f.write_str(std::str::from_utf8(&[character as u8]).unwrap())?;
        }
        f.write_str(".")?;
        f.write_str(self.group.as_str())
    }
}

/// Container which can hold multiple elements of tag blocks.
#[derive(Default, Clone)]
pub struct Reflexive<T: TagBlockFn> {
    pub blocks: Vec<T>
}

use std::vec::IntoIter;
use std::slice::{Iter, IterMut};

impl<T: TagBlockFn> IntoIterator for Reflexive<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        self.blocks.into_iter()
    }
}

impl<'a, T: TagBlockFn> IntoIterator for &'a Reflexive<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        self.blocks.iter()
    }
}

impl<'a, T: TagBlockFn> IntoIterator for &'a mut Reflexive<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> IterMut<'a, T> {
        self.blocks.iter_mut()
    }
}

impl<T: TagBlockFn + PartialEq> PartialEq for Reflexive<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.blocks == rhs.blocks
    }
}

impl<T: TagBlockFn> ReflexiveFn for Reflexive<T> {
    fn len(&self) -> usize {
        self.blocks.len()
    }
    fn block_at_index(&self, index: usize) -> &dyn TagBlockFn {
        &self.blocks[index]
    }
    fn block_at_index_mut(&mut self, index: usize) -> &mut dyn TagBlockFn {
        &mut self.blocks[index]
    }
}

impl<T: TagBlockFn> Index<usize> for Reflexive<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks[index]
    }
}

impl<T: TagBlockFn> IndexMut<usize> for Reflexive<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.blocks[index]
    }
}

impl<T: TagBlockFn> Reflexive<T> {
    /// Construct a new `Reflexive` with a vector of blocks.
    pub fn new(vec: Vec<T>) -> Reflexive<T> {
        Reflexive { blocks: vec }
    }
}
