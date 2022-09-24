//! Functions for enumerating tag fields on runtime.

use std::any::Any;
use std::any::TypeId;

use crate::types::*;

#[cfg(test)]
mod tests;

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

/// General interface for accessing blocks from an [Reflexive] type of an unknown block type.
pub trait ReflexiveFn {
    /// Get the length of the array.
    fn len(&self) -> usize;

    /// Get the block at the index or panic if out of bounds.
    fn block_at_index(&self, index: usize) -> &dyn TagBlockFn;

    /// Get the mutable block at the index or panic if out of bounds.
    fn block_at_index_mut(&mut self, index: usize) -> &mut dyn TagBlockFn;
}

/// General interface for tag group parsing.
pub trait TagGroupFn where Self: Sized {
    /// Get a tag group from the [`FourCC`] signature or `None` if the FourCC is unrecognized.
    fn from_fourcc(fourcc: FourCC) -> Option<Self>;

    /// Get the internal FourCC of the tag group.
    fn as_fourcc(&self) -> FourCC;

    /// Get the name of the tag group used in file extensions.
    fn as_str(&self) -> &'static str;

    /// Get a tag group from a string or `None` if the string is unrecognized.
    fn from_str(str: &str) -> Option<Self>;

    /// Get the `None` value of this tag group.
    fn none() -> Self;
}

/// Tag field of some kind
pub enum TagFieldValue<'a> {
    /// Value
    Value(FieldReference<&'a dyn Any>),

    /// Value (mutable)
    MutableValue(FieldReference<&'a mut dyn Any>),

    /// Block array
    Array(&'a dyn ReflexiveFn),

    /// Mutable block array
    MutableArray(&'a mut dyn ReflexiveFn),

    /// Bounds
    Bounds(&'a dyn BoundsFn),

    /// Bounds (mutable)
    MutableBounds(&'a mut dyn BoundsFn),
}

/// Reference to a value in a tag.
pub struct FieldReference<T> {
    /// Field being accessed.
    pub field: T
}

/// Field of a tag.
pub struct TagField<'a> {
    /// Field being accessed
    pub field: TagFieldValue<'a>,

    /// Name of the field.
    pub name: &'static str,

    /// Description of the field.
    pub comment: &'static str
}

impl FieldReference<&mut dyn Any> {
    /// Downcast the value into a [ValueReferenceMut].
    pub fn get_value(&mut self) -> ValueReferenceMut {
        macro_rules! attempt_downcast {
            ($t:ty, $into:tt) => {{
                if TypeId::of::<$t>() == (*self.field).type_id() {
                    return ValueReferenceMut::$into(self.field.downcast_mut::<$t>().unwrap())
                }
            }}
        }

        attempt_downcast!(i8, Int8);
        attempt_downcast!(i16, Int16);
        attempt_downcast!(i32, Int32);
        attempt_downcast!(u8, UInt8);
        attempt_downcast!(u16, UInt16);
        attempt_downcast!(u32, UInt32);
        attempt_downcast!(f32, Float32);

        attempt_downcast!(ColorAHSV, ColorAHSV);
        attempt_downcast!(ColorARGB, ColorARGB);
        attempt_downcast!(ColorARGBInt, ColorARGBInt);
        attempt_downcast!(ColorHSV, ColorHSV);
        attempt_downcast!(ColorRGB, ColorRGB);
        attempt_downcast!(ColorRGBInt, ColorRGBInt);
        attempt_downcast!(Euler2D, Euler2D);
        attempt_downcast!(Euler3D, Euler3D);
        attempt_downcast!(Matrix, Matrix);
        attempt_downcast!(Plane2D, Plane2D);
        attempt_downcast!(Plane3D, Plane3D);
        attempt_downcast!(Point2D, Point2D);
        attempt_downcast!(Point2DInt, Point2DInt);
        attempt_downcast!(Point3D, Point3D);
        attempt_downcast!(Quaternion, Quaternion);
        attempt_downcast!(Rectangle, Rectangle);
        attempt_downcast!(String32, String32);
        attempt_downcast!(Vector2D, Vector2D);
        attempt_downcast!(Vector3D, Vector3D);

        attempt_downcast!(crate::engines::h1::TagReference, H1TagReference);

        unreachable!()
    }
}

impl FieldReference<&dyn Any> {
    /// Downcast the value into a [ValueReference].
    pub fn get_value(&self) -> ValueReference {
        macro_rules! attempt_downcast {
            ($t:ty, $into:tt) => {{
                if TypeId::of::<$t>() == (*self.field).type_id() {
                    return ValueReference::$into(self.field.downcast_ref::<$t>().unwrap())
                }
            }}
        }

        attempt_downcast!(i8, Int8);
        attempt_downcast!(i16, Int16);
        attempt_downcast!(i32, Int32);
        attempt_downcast!(u8, UInt8);
        attempt_downcast!(u16, UInt16);
        attempt_downcast!(u32, UInt32);
        attempt_downcast!(f32, Float32);

        attempt_downcast!(ColorAHSV, ColorAHSV);
        attempt_downcast!(ColorARGB, ColorARGB);
        attempt_downcast!(ColorARGBInt, ColorARGBInt);
        attempt_downcast!(ColorHSV, ColorHSV);
        attempt_downcast!(ColorRGB, ColorRGB);
        attempt_downcast!(ColorRGBInt, ColorRGBInt);
        attempt_downcast!(Euler2D, Euler2D);
        attempt_downcast!(Euler3D, Euler3D);
        attempt_downcast!(Matrix, Matrix);
        attempt_downcast!(Plane2D, Plane2D);
        attempt_downcast!(Plane3D, Plane3D);
        attempt_downcast!(Point2D, Point2D);
        attempt_downcast!(Point2DInt, Point2DInt);
        attempt_downcast!(Point3D, Point3D);
        attempt_downcast!(Quaternion, Quaternion);
        attempt_downcast!(Rectangle, Rectangle);
        attempt_downcast!(String32, String32);
        attempt_downcast!(Vector2D, Vector2D);
        attempt_downcast!(Vector3D, Vector3D);

        attempt_downcast!(crate::engines::h1::TagReference, H1TagReference);

        unreachable!()
    }
}

/// Trait for accessing Bounds fields.
pub trait BoundsFn {
    /// Get the lower bound for the bounds.
    fn get_lower(&self) -> FieldReference<&dyn Any>;

    /// Get the upper bound for the bounds.
    fn get_upper(&self) -> FieldReference<&dyn Any>;

    /// Get the lower bound for the bounds.
    fn get_lower_mut(&mut self) -> FieldReference<&mut dyn Any>;

    /// Get the upper bound for the bounds.
    fn get_upper_mut(&mut self) -> FieldReference<&mut dyn Any>;
}

impl<T: Any> BoundsFn for Bounds<T> {
    fn get_lower(&self) -> FieldReference<&dyn Any> {
        FieldReference { field: &self.lower }
    }
    fn get_upper(&self) -> FieldReference<&dyn Any> {
        FieldReference { field: &self.upper }
    }
    fn get_lower_mut(&mut self) -> FieldReference<&mut dyn Any> {
        FieldReference { field: &mut self.lower }
    }
    fn get_upper_mut(&mut self) -> FieldReference<&mut dyn Any> {
        FieldReference { field: &mut self.upper }
    }
}

/// General interface for dynamically enumerating the tag structure at runtime.
pub trait TagBlockFn: Any {
    /// Get the number of fields.
    fn field_count(&self) -> usize;

    /// Get the field at the given index. Panics if it is out of bounds.
    fn field_at_index(&self, index: usize) -> TagField;

    /// Get the mutable field at the given index. Panics if it is out of bounds.
    fn field_at_index_mut(&mut self, index: usize) -> TagField;
}

impl Index<usize> for &dyn ReflexiveFn {
    type Output = dyn TagBlockFn;
    fn index(&self, index: usize) -> &Self::Output {
        self.block_at_index(index)
    }
}

impl Index<usize> for &mut dyn ReflexiveFn {
    type Output = dyn TagBlockFn;
    fn index(&self, index: usize) -> &Self::Output {
        self.block_at_index(index)
    }
}

impl IndexMut<usize> for &mut dyn ReflexiveFn {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.block_at_index_mut(index)
    }
}


/// Typed reference to some value.
pub enum ValueReference<'a> {
    Int8(&'a i8),
    Int16(&'a i16),
    Int32(&'a i32),
    UInt8(&'a u8),
    UInt16(&'a u16),
    UInt32(&'a u32),
    Float32(&'a f32),

    ColorAHSV(&'a ColorAHSV),
    ColorARGB(&'a ColorARGB),
    ColorARGBInt(&'a ColorARGBInt),
    ColorHSV(&'a ColorHSV),
    ColorRGB(&'a ColorRGB),
    ColorRGBInt(&'a ColorRGBInt),
    Euler2D(&'a Euler2D),
    Euler3D(&'a Euler3D),
    Matrix(&'a Matrix),
    Plane2D(&'a Plane2D),
    Plane3D(&'a Plane3D),
    Point2D(&'a Point2D),
    Point2DInt(&'a Point2DInt),
    Point3D(&'a Point3D),
    Quaternion(&'a Quaternion),
    Rectangle(&'a Rectangle),
    String32(&'a String32),
    Vector2D(&'a Vector2D),
    Vector3D(&'a Vector3D),

    H1TagReference(&'a crate::engines::h1::TagReference)
}

/// Typed mutable reference to some value.
pub enum ValueReferenceMut<'a> {
    Int8(&'a mut i8),
    Int16(&'a mut i16),
    Int32(&'a mut i32),
    UInt8(&'a mut u8),
    UInt16(&'a mut u16),
    UInt32(&'a mut u32),
    Float32(&'a mut f32),

    ColorAHSV(&'a mut ColorAHSV),
    ColorARGB(&'a mut ColorARGB),
    ColorARGBInt(&'a mut ColorARGBInt),
    ColorHSV(&'a mut ColorHSV),
    ColorRGB(&'a mut ColorRGB),
    ColorRGBInt(&'a mut ColorRGBInt),
    Euler2D(&'a mut Euler2D),
    Euler3D(&'a mut Euler3D),
    Matrix(&'a mut Matrix),
    Plane2D(&'a mut Plane2D),
    Plane3D(&'a mut Plane3D),
    Point2D(&'a mut Point2D),
    Point2DInt(&'a mut Point2DInt),
    Point3D(&'a mut Point3D),
    Quaternion(&'a mut Quaternion),
    Rectangle(&'a mut Rectangle),
    String32(&'a mut String32),
    Vector2D(&'a mut Vector2D),
    Vector3D(&'a mut Vector3D),

    H1TagReference(&'a mut crate::engines::h1::TagReference)
}

/// Create a path based on [`TagReference`]'s rules except for detecting if a path consists of only backslashes.
///
/// This function does not take paths with extensions, as it will not be able to detect if a path consists of only
/// directory separators if an extension is present.
fn normalize_path(path: &str) -> ErrorMessageResult<String> {
    let path_bytes = path.as_bytes();
    let mut new_path = String::new();
    new_path.reserve(path_bytes.len());

    let mut last_character = None;
    for c in path.as_bytes() {
        let mut character = *c as char;

        // Allow any lowercase characters or numeric characters
        if character.is_ascii_lowercase() || character.is_ascii_digit() {
            ()
        }

        // Convert path separators
        else if character == std::path::MAIN_SEPARATOR || character == HALO_DIRECTORY_SEPARATOR || character == '/' {
            // Strip double path separators and leading path separator
            if last_character == Some(HALO_DIRECTORY_SEPARATOR) || last_character == None {
                continue
            }

            // Also check that the last directory is not "." or ".."
            else if new_path.ends_with("\\.") {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_invalid_directory"), new_path=path, directory=".")))
            }
            else if new_path.ends_with("\\..") {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_invalid_directory"), new_path=path, directory="..")))
            }

            // Otherwise, Halo uses backslashes as its path separator
            character = HALO_DIRECTORY_SEPARATOR;
        }

        // Make uppercase characters lowercase
        else if character.is_ascii_uppercase() {
            character = character.to_ascii_lowercase();
        }

        // Ban these characters
        else if ['<', '>', ':', '"', /*'/',*/ '|', '?', '*', '\x00'].contains(&character) {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_has_restricted_character"), new_path=path, character=character)))
        }

        // Ban non-ascii
        else if !character.is_ascii() {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_not_ascii"), new_path=path)))
        }

        new_path.push(character);
        last_character = Some(character);
    }

    // Check if the path starts with '.\' or '..\'
    if new_path.starts_with(".\\") {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_invalid_directory"), new_path=path, directory=".")))
    }
    else if new_path.starts_with("..\\") {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_invalid_directory"), new_path=path, directory="..")))
    }

    // If the new path is empty, but we had text before that, we just broke the reference when we cleaned it, and this is an error.
    if new_path.is_empty() && !path.is_empty() {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_empty_when_non_empty"), new_path=path)));
    }

    Ok(new_path)
}

/// Tag reference that describes a tag.
///
/// For all functions that set a path, the path must be a valid path for Windows. The following characters are
/// restricted by Windows:
///
/// - `/`    (forward slash - will be replaced with `\`)
/// - `<`    (less than)
/// - `>`    (greater than)
/// - `:`    (colon)
/// - `"`    (double quote)
/// - `|`    (vertical bar or pipe)
/// - `?`    (question mark)
/// - `*`    (asterisk)
/// - `\x00` (null terminator for C strings)
/// - Non ASCII characters
///
/// In addition, these paths are not allowed:
///
/// - Paths consisting of only directory separators (e.g. `\\\\\.weapon`)
///   - The HEK is inconsistent with how it handles these paths, and Invader removes paths that begin with path
///     separators, thus these paths cannot be handled sanely.
/// - Paths containing directories named `.` or `..`
///   - While valid as paths, these present a security issue where files outside a tags directory can be referenced
///     without a user creating symbolic links, mount points, etc., and they are less predictable for virtual tags
///     directories. Invader supports using multiple tags directories simultaneously, so that feature should be used,
///     instead.
///
/// Also, all characters will be made lowercase, and the native path separator will be replaced with a backslash (`\`).
/// Paths with consecutive directory separators will also have these separators deduped, and paths that start with a
/// directory separator will have these separators removed.
///
/// If an invalid path is used, the string will not be written, and an [`Err`] will be returned with a message.
///
/// Note that the [`to_string`](std::string::ToString::to_string) function and [`Display`](std::fmt::Display) will
/// display native path separators. Use [`get_path_with_extension`](TagReference::get_path_with_extension) if you want
/// to get a non-OS dependent path, or use [`get_path_without_extension`](TagReference::get_path_without_extension) if
/// the extension is unneeded, as this avoids allocating a new string.
#[derive(Clone, Default, PartialEq, Debug)]
pub struct TagReference<T: TagGroupFn> {
    group: T,
    path: String
}
impl<T: TagGroupFn + Copy + Default> TagReference<T> {
    /// Get the path concatenated with the file extension.
    ///
    /// This will return the path with Halo separators (i.e. backslashes).
    pub fn get_path_with_extension(&self) -> String {
        format!("{}.{}", self.path, self.group.as_str())
    }

    /// Get the path without the file extension.
    ///
    /// This will return the path with Halo separators (i.e. backslashes).
    pub fn get_path_without_extension(&self) -> &str {
        &self.path
    }

    /// Convert to a relative filesystem path.
    ///
    /// This is a convenience function for using [`to_string`](std::string::ToString::to_string) and wrapping the
    /// result in a [`PathBuf`].
    pub fn get_relative_fs_path(&self) -> PathBuf {
        PathBuf::from(self.to_string())
    }

    /// Set the full path also including the tag group.
    ///
    /// If the path is invalid for a `TagReference`, an [`Err`] is returned.
    pub fn set_full_path(&mut self, path: &str) -> ErrorMessageResult<()> {
        // Find the extension.
        let new_base_length = match path.rfind('.') {
            Some(n) => n,
            None => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_missing_extension"), path=path)))
        };

        // Get the group
        let (base_path, extension) = path.split_at(new_base_length);
        let group = match T::from_str(&extension[1..]) {
            Some(n) => n,
            None => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_group_invalid"), potential_group=extension)))
        };

        // Set it
        self.set_path_without_extension(base_path)?;
        self.group = group;

        Ok(())
    }

    /// Set the path, maintaining the tag group.
    ///
    /// If the path is invalid for a `TagReference`, an [`Err`] is returned.
    pub fn set_path_without_extension(&mut self, path: &str) -> ErrorMessageResult<()> {
        // Set it
        self.path = normalize_path(path)?;

        Ok(())
    }

    /// Create a `TagReference` from a path containing an extension that corresponds to a tag group.
    ///
    /// If the path is invalid for a `TagReference` or the extension is invalid or nonexistent, an [`Err`] is returned.
    pub fn from_full_path(path: &str) -> ErrorMessageResult<TagReference<T>> {
        let mut reference = TagReference::default();
        reference.set_full_path(path)?;
        Ok(reference)
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
    /// See [`match_pattern`] for more information on how this function works.
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        match_pattern(&self.get_path_with_extension(), pattern)
    }

    /// Get the group.
    pub fn get_group(&self) -> T {
        self.group
    }

    /// Set the group
    pub fn set_group(&mut self, group: T) {
        self.group = group
    }
}

impl<T: TagGroupFn> fmt::Display for TagReference<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for c in self.path.chars() {
            let mut character = c;
            if c == HALO_DIRECTORY_SEPARATOR {
                character = std::path::MAIN_SEPARATOR;
            }
            f.write_str(std::str::from_utf8(&[character as u8]).unwrap())?;
        }

        f.write_str(".")?;
        f.write_str(self.group.as_str())
    }
}
