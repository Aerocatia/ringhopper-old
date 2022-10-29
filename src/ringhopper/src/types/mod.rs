//! Types of fields used in tag structs.

use std::cmp::PartialEq;
use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use std::fmt;

pub mod tag;
pub use self::tag::*;

mod trig;
pub use self::trig::*;

mod color;
pub use self::color::*;

use crate::error::*;
use ringhopper_proc::*;

/// FourCC type (aliased to a 32-bit unsigned integer)
pub type FourCC = u32;

/// A block of data that doesn't have any fields directly attributed to it.
pub type Data = Vec<u8>;

/// Trait for accessing tag enums.
pub trait TagEnumFn {
    /// Get the numeric representation of the enum.
    fn into_u16(self) -> u16 where Self: Sized;

    /// Convert the number into an enum.
    ///
    /// Return an [`Err`] if the value is out of bounds for the enum.
    fn from_u16(input_value: u16) -> ErrorMessageResult<Self> where Self: Sized;

    /// Get all options.
    fn options() -> &'static [&'static str];

    /// Get all options as their display representation.
    fn options_pretty() -> &'static [&'static str];

    /// Get the string representation of the enum.
    fn as_str(self) -> &'static str where Self: Sized {
        Self::options()[self.into_u16() as usize]
    }

    /// Get the display string representation of the enum.
    fn as_str_pretty(self) -> &'static str where Self: Sized {
        Self::options_pretty()[self.into_u16() as usize]
    }
}

/// Halo directory separator used in tag paths.
///
/// This corresponds to a Windows path separator.
pub const HALO_DIRECTORY_SEPARATOR: char = '\\';

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

/// Point with two X/Y shorts.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Point2DInt {
    /// X value.
    pub x: i16,

    /// Y value.
    pub y: i16
}

/// Point with two X/Y unsigned shorts.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Point2DUInt {
    /// X value.
    pub x: u16,

    /// Y value.
    pub y: u16
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
        if (p == s && p != '*') || p == '?' || ((p == '/' || p == HALO_DIRECTORY_SEPARATOR || p == std::path::MAIN_SEPARATOR) && (s == '/' || s == HALO_DIRECTORY_SEPARATOR || s == std::path::MAIN_SEPARATOR)) {
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

/// Compute log2 on a 16-bit unsigned integer, rounding down.
///
/// NOTE: THIS SHOULD BE REPLACED WHEN log_int BECOMES STABILIZED!
pub(crate) fn log2_u16(input: u16) -> u16 {
    debug_assert!(input > 0, "cannot log2 zero");

    let mut v = 0;
    let mut i = input / 2;

    while i > 0 {
        i /= 2;
        v += 1;
    }

    v
}
