use std::fmt;

mod trig;
pub use self::trig::*;

mod color;
pub use self::color::*;

mod tag;
pub use self::tag::*;

/// String with a maximum character length of 31 characters plus a null terminator.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct String32 {
    bytes: [u8; 32],
    length: usize
}

impl String32 {
    /// Convert into a Rust `str` reference with the same lifespan.
    pub fn to_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[0..self.length]).unwrap()
    }

    /// Convert 32 `u8` bytes into a `String32`. The input must be a valid, null terminated UTF-8 string.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<String32, &'static str> {
        let mut length: usize = 0;

        for b in bytes {
            if *b == 0 {
                break;
            }
            length += 1;
        }

        if length == bytes.len() {
            Err("string overflow")
        }
        else if !std::str::from_utf8(&bytes[0..length]).is_ok() {
            Err("string data is not a valid string")
        }
        else {
            // Clean everything after the string
            let mut bytes_copy = *bytes;
            for i in &mut bytes_copy[length..] {
                *i = 0
            }
            Ok(String32 { bytes: bytes_copy, length: length })
        }
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
    pub point1: Point2DInt,
    pub point2: Point2DInt
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
    ///
    /// # Example code:
    ///
    /// ```rust
    /// use invader::TagReference;
    ///
    /// let mut reference = TagReference::<invader::hce::TagGroup>::from_path_with_extension("weapons\\assault rifle\\assault rifle.weapon").unwrap();
    ///
    /// reference.set_path_without_extension("weapons\\pistol\\pistol");
    /// ```
    pub fn set_path_without_extension(&mut self, path: &str) -> Result<(), &'static str> {
        let mut new_path = path.to_owned();

        // Paths must be ASCII
        if !new_path.is_ascii() {
            return Err("path is non-ASCII")
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
        if new_path.contains(&['<', '>', ':', '"', '/', '|', '?', '*']) {
            return Err("path contains restricted characters")
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
    pub fn from_path_with_extension(path: &str) -> Result<TagReference<T>, &'static str> {
        let pos = match path.rfind('.') {
            Some(n) => n,
            None => return Err("path did not have a file extension")
        };

        let group = match T::from_str(&path[pos+1..]) {
            Some(n) => n,
            None => return Err("extension does not correspond to a tag group")
        };

        TagReference::from_path_and_group(&path[..pos], group)
    }

    /// Create a `TagReference` from a path and group.
    ///
    /// If the path is invalid for a `TagReference`, an [Err] is returned.
    pub fn from_path_and_group(path: &str, group: T) -> Result<TagReference<T>, &'static str> {
        let mut reference = TagReference { path: String::default(), group: group };
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
