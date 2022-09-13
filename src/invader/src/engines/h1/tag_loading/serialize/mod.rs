#[cfg(test)]
mod tests;

use crate::error::*;
use crate::types::*;
use crate::types::tag::TagGroupFn;
use crate::engines::h1::types::{TagGroup, TagReference};
use crate::types::tag::TagBlockFn;
use strings::get_compiled_string;

use std::any::Any;

/// FourCC "blam" used for tag file headers.
pub const BLAM_FOURCC: u32 = 0x626C616D;

/// Header used for tag files
pub struct TagFileHeader {
    /// Old tag ID. Unread.
    pub old_tag_id: u32,

    /// Old tag name. Unread.
    pub old_tag_name: String32,

    /// Tag group. Must be verified.
    pub tag_group: TagGroup,

    /// CRC32 checksum of the data after the header. Used for building cache files and possibly other things.
    pub crc32: u32,

    /// Equals 0x40, the size of the header. Probably unread.
    pub header_length: u32,

    /// Version of the tag group. Must be verified. See [TagFileHeader::version_for_group].
    pub tag_group_version: u16,

    /// Equals 255. Probably unread.
    pub something_255: u16,

    /// Equals [BLAM_FOURCC]. Must be verified.
    pub blam_fourcc: u32
}

impl TagFileHeader {
    /// Validate the header, returning an error with an explanation if it is invalid.
    pub fn validate(&self) -> ErrorMessageResult<()> {
        match self.validate_encapsulate() {
            Ok(n) => Ok(n),
            Err(n) => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.tag.header.error"), reason=n.to_string())))
        }
    }

    fn validate_encapsulate(&self) -> ErrorMessageResult<()> {
        if self.blam_fourcc != BLAM_FOURCC {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.tag.header.error_reason_blam_invalid"), blam=BLAM_FOURCC, other=self.blam_fourcc)))
        }

        let expected_version = TagFileHeader::version_for_group(self.tag_group);
        if self.tag_group_version != expected_version {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.tag.header.error_reason_version_unsupported"), group=self.tag_group, version=expected_version, other=self.tag_group_version)))
        }

        Ok(())
    }

    /// Get the version supported for a tag group.
    pub fn version_for_group(group: TagGroup) -> u16 {
        match group {
            TagGroup::Actor => 2,
            TagGroup::ModelAnimations => 4,
            TagGroup::Biped => 3,
            TagGroup::Bitmap => 7,
            TagGroup::Contrail => 3,
            TagGroup::Effect => 4,
            TagGroup::Equipment => 2,
            TagGroup::Item => 2,
            TagGroup::ItemCollection => 0,
            TagGroup::DamageEffect => 6,
            TagGroup::LensFlare => 2,
            TagGroup::Light => 3,
            TagGroup::SoundLooping => 3,
            TagGroup::GBXModel => 5,
            TagGroup::Globals => 3,
            TagGroup::Model => 4,
            TagGroup::ModelCollisionGeometry => 10,
            TagGroup::Particle => 2,
            TagGroup::ParticleSystem => 4,
            TagGroup::Physics => 4,
            TagGroup::Placeholder => 2,
            TagGroup::PreferencesNetworkGame => 2,
            TagGroup::Projectile => 5,
            TagGroup::ScenarioStructureBSP => 5,
            TagGroup::Scenario => 2,
            TagGroup::ShaderEnvironment => 2,
            TagGroup::Sound => 4,
            TagGroup::ShaderModel => 2,
            TagGroup::ShaderTransparentWater => 2,
            TagGroup::CameraTrack => 2,
            TagGroup::Unit => 2,
            TagGroup::VirtualKeyboard => 2,
            TagGroup::Weapon => 2,
            TagGroup::WeaponHUDInterface => 2,
            _ => 1
        }
    }
}

/// Serialization implementation for tags in tag format using the normal endian (big endian).
pub trait TagSerialize {
    /// Get the size of the data
    fn tag_size() -> usize where Self: Sized;

    /// Serialize the data into tag format, returning an error on failure (except for out-of-bounds and allocation errors which will panic).
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()>;

    /// Deserialize the data from tag format, returning an error on failure (except for allocation errors which will panic).
    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<Self> where Self: Sized;

    /// Get the size of the instance.
    fn instance_tag_size(&self) -> usize where Self: Sized {
        Self::tag_size()
    }

    /// Deserialize into an instance.
    fn instance_from_tag(&self, data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<Self> where Self: Sized {
        Self::from_tag(data, at, struct_end, cursor)
    }
}

/// Serialization implementation for tags in tag format using little endian.
pub trait TagSerializeSwapped: TagSerialize {
    /// Serialize the data into tag format in little endian, returning an error on failure (except for out-of-bounds and allocation errors which will panic).
    fn into_tag_swapped(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> where Self: Sized;

    /// Deserialize the data from tag format from little endian, returning an error on failure (except for allocation errors which will panic).
    fn from_tag_swapped(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<Self> where Self: Sized;
}

macro_rules! sizeof {
    ($t:ty) => {
        std::mem::size_of::<$t>()
    }
}

/// Check if a field fits in the given struct bounds.
///
/// Return `Ok` if this is true or `Err` if not.
///
/// Panics if debug assertions are enabled and data in the struct goes outside of the input struct size, as this is a programming error.
///
/// `size` is the size of the field, `at` is the offset of the field, `struct_end` is the length of the struct the field is in, `vec_size` is the size of the vector holding the data
fn fits(size: usize, at: usize, struct_end: usize, vec_size: usize) -> ErrorMessageResult<()> {
    let end = match at.checked_add(size) {
        Some(n) => n,
        None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.serialize.error_architecture_limit_exceeded")))
    };

    // If data is out of the struct bounds, then this is a programming error rather than bad tag data as it means our struct size is wrong.
    debug_assert!(end <= struct_end, "Data is outside of the struct (this is a bug!) - (0x{at:08X} [offset] + 0x{size:08X} [size] = 0x{end:08X} [end]) <= 0x{struct_end:08X} [struct_end]", at=at, size=size, end=end, struct_end=struct_end);

    // If we're outside of the data bounds, fail.
    if end > vec_size {
        Err(ErrorMessage::StaticString("Data is out of bounds."))
    }
    else {
        Ok(())
    }
}

/// Checks if data fits in the given tag.
///
/// Returns `Ok` if this is true or `Err` if not.
///
/// `size` is the size of the data, `at` is the offset of the data, `vec_size` is the size of the vector holding the data
fn fits_extra_data(size: usize, at: usize, vec_size: usize) -> ErrorMessageResult<()> {
    let data_end = match at.checked_add(size) {
        Some(n) => n,
        None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.serialize.error_architecture_limit_exceeded")))
    };

    if data_end > vec_size {
        Err(ErrorMessage::StaticString("Data is out of bounds."))
    }
    else {
        Ok(())
    }
}

macro_rules! serialize_for_primitive {
    ($t:ty) => {
        impl TagSerialize for $t {
            fn tag_size() -> usize {
                sizeof!($t)
            }

            fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {
                const SIZE: usize = sizeof!($t);
                debug_assert!(fits(SIZE, at, struct_end, data.len()).is_ok());
                let bytes = self.to_be_bytes();
                data[at..at + SIZE].copy_from_slice(&bytes[..]);
                Ok(())
            }

            fn from_tag(data: &[u8], at: usize, struct_end: usize, _: &mut usize) -> ErrorMessageResult<Self> {
                use std::convert::TryInto;

                const SIZE: usize = sizeof!($t);
                fits(SIZE, at, struct_end, data.len())?;

                let bytes: [u8; SIZE] = data[at..at + SIZE].try_into().unwrap();
                Ok(<$t>::from_be_bytes(bytes))
            }
        }

        impl TagSerializeSwapped for $t {
            fn into_tag_swapped(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {
                const SIZE: usize = sizeof!($t);
                debug_assert!(fits(SIZE, at, struct_end, data.len()).is_ok());
                let bytes = self.to_le_bytes();
                data[at..at + SIZE].copy_from_slice(&bytes[..]);
                Ok(())
            }

            fn from_tag_swapped(data: &[u8], at: usize, struct_end: usize, _: &mut usize) -> ErrorMessageResult<Self> {
                use std::convert::TryInto;

                const SIZE: usize = sizeof!($t);
                fits(SIZE, at, struct_end, data.len())?;

                let bytes: [u8; SIZE] = data[at..at + SIZE].try_into().unwrap();
                Ok(<$t>::from_le_bytes(bytes))
            }
        }
    };
}

serialize_for_primitive!(i8);
serialize_for_primitive!(i16);
serialize_for_primitive!(i32);
serialize_for_primitive!(u8);
serialize_for_primitive!(u16);
serialize_for_primitive!(u32);
serialize_for_primitive!(f32);

macro_rules! add_sizeof_together {
    // base case
    ($instance:tt, $field:tt) => ($instance.$field.instance_tag_size());

    // stuff
    ($instance:tt, $field:tt, $($fields:tt), +) => (
        add_sizeof_together!($instance, $field) + add_sizeof_together!($instance, $($fields), +)
    );
}

macro_rules! into_tag {
    // base case
    ($data:expr, $at:tt, $struct_end:tt, $instance:tt, $field:tt) => ({
        let size = $instance.$field.instance_tag_size();
        $instance.$field.into_tag($data, $at, $struct_end)?;
        $at += size;
    });

    // stuff
    ($data:expr, $at:tt, $struct_end:tt, $instance:tt, $field:tt, $($fields:tt), +) => (
        into_tag!($data, $at, $struct_end, $instance, $field);
        into_tag!($data, $at, $struct_end, $instance, $($fields), +);
    );
}

macro_rules! from_tag {
    // base case
    ($data:expr, $at:tt, $struct_end:tt, $instance:tt, $field:tt) => ({
        let size = $instance.$field.instance_tag_size();
        $instance.$field = $instance.$field.instance_from_tag($data, $at, $struct_end, &mut 0)?;
        $at += size;
    });

    // stuff
    ($data:expr, $at:tt, $struct_end:tt, $instance:tt, $field:tt, $($fields:tt), +) => (
        from_tag!($data, $at, $struct_end, $instance, $field);
        from_tag!($data, $at, $struct_end, $instance, $($fields), +);
    );
}

macro_rules! serialize_for_struct {
    ($t:ty, $($fields:tt), +) => {
        impl TagSerialize for $t {
            fn tag_size() -> usize {
                // Add everything together
                let instance = Self::default();
                add_sizeof_together!(instance, $($fields), +)
            }

            fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {
                // Verify
                let size: usize = self.instance_tag_size();
                debug_assert!(fits(size, at, struct_end, data.len()).is_ok());

                // Now go through each member one at a time
                let mut at_start = at;
                into_tag!(data, at_start, struct_end, self, $($fields), +);

                // Verify that we have the correct offset
                debug_assert_eq!(at + size, at_start);

                Ok(())
            }

            fn from_tag(data: &[u8], at: usize, struct_end: usize, _: &mut usize) -> ErrorMessageResult<Self> {
                let mut instance = Self::default();

                // Verify
                let size: usize = instance.instance_tag_size();
                fits(size, at, struct_end, data.len())?;

                // Now go through each member one at a time
                let mut at_start = at;
                from_tag!(data, at_start, struct_end, instance, $($fields), +);

                // Verify that we have the correct offset
                debug_assert_eq!(at + size, at_start);

                // Done!
                Ok(instance)
            }
        }
    }
}

// For these, we just need to specify the order
serialize_for_struct!(ColorAHSV, a, h, s, v);
serialize_for_struct!(ColorARGB, a, r, g, b);
serialize_for_struct!(ColorHSV, h, s, v);
serialize_for_struct!(ColorRGB, r, g, b);
serialize_for_struct!(Euler2D, y, p);
serialize_for_struct!(Euler3D, y, p, r);
serialize_for_struct!(Plane2D, vector, d);
serialize_for_struct!(Plane3D, vector, d);
serialize_for_struct!(Point2D, x, y);
serialize_for_struct!(Point2DInt, x, y);
serialize_for_struct!(Point3D, x, y, z);
serialize_for_struct!(Quaternion, x, y, z, w);
serialize_for_struct!(Rectangle, top, left, bottom, right);
serialize_for_struct!(Vector2D, x, y);
serialize_for_struct!(Vector3D, x, y, z);

impl<T: TagSerialize> TagSerialize for Bounds<T> {
    fn tag_size() -> usize {
        T::tag_size() * 2
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {
        debug_assert!(fits(Self::tag_size(), at, struct_end, data.len()).is_ok());

        self.lower.into_tag(data, at, struct_end)?;
        self.upper.into_tag(data, at + T::tag_size(), struct_end)?;

        Ok(())
    }
    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<Self> {
        fits(Self::tag_size(), at, struct_end, data.len())?;

        let lower = T::from_tag(data, at, struct_end, cursor)?;
        let upper = T::from_tag(data, at + T::tag_size(), struct_end, cursor)?;

        Ok(Self { lower, upper })
    }
}

// These convert to 32-bit integers, so we can serialize them as such
impl TagSerialize for ColorARGBInt {
    fn tag_size() -> usize {
        u32::tag_size()
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, offset: usize) -> ErrorMessageResult<()> {
        self.to_a8r8g8b8().into_tag(data, at, offset)
    }
    fn from_tag(data: &[u8], at: usize, offset: usize, cursor: &mut usize) -> ErrorMessageResult<Self> {
        Ok(Self::from_a8r8g8b8(u32::from_tag(data, at, offset, cursor)?))
    }
}

impl TagSerialize for ColorRGBInt {
    fn tag_size() -> usize {
        u32::tag_size()
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, offset: usize) -> ErrorMessageResult<()> {
        ColorARGBInt::from(*self).to_a8r8g8b8().into_tag(data, at, offset)
    }
    fn from_tag(data: &[u8], at: usize, offset: usize, cursor: &mut usize) -> ErrorMessageResult<Self> {
        let argb = ColorARGBInt::from_a8r8g8b8(u32::from_tag(data, at, offset, cursor)?);
        Ok(Self { r: argb.r, g: argb.g, b: argb.b })
    }
}

// This is a simple 32 byte array
impl TagSerialize for String32 {
    fn tag_size() -> usize {
        32
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize)  -> ErrorMessageResult<()> {
        const SIZE: usize = 32;
        debug_assert!(fits(SIZE, at, struct_end, data.len()).is_ok());
        data[at..at + SIZE].copy_from_slice(&self.bytes[..]);
        Ok(())
    }
    fn from_tag(data: &[u8], at: usize, struct_end: usize, _: &mut usize) -> ErrorMessageResult<Self> {
        const SIZE: usize = 32;
        use std::convert::TryInto;

        fits(SIZE, at, struct_end, data.len())?;
        let bytes: [u8; SIZE] = data[at..at + SIZE].try_into().unwrap();
        Self::from_bytes(bytes)
    }
}

const VECTOR_STRUCT_SIZE: usize = sizeof!(f32) * 3;
const MATRIX_STRUCT_SIZE: usize = VECTOR_STRUCT_SIZE * 3;

impl TagSerialize for Matrix {
    fn tag_size() -> usize {
        MATRIX_STRUCT_SIZE
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize)  -> ErrorMessageResult<()> {
        debug_assert!(fits(MATRIX_STRUCT_SIZE, at, struct_end, data.len()).is_ok());

        // Go through each vector
        for i in 0..3 {
            self.vectors[i].into_tag(data, at + i * VECTOR_STRUCT_SIZE, struct_end)?;
        }

        Ok(())
    }
    fn from_tag(data: &[u8], at: usize, mut struct_end: usize, _: &mut usize) -> ErrorMessageResult<Self> {
        // Does the base struct fit?
        fits(MATRIX_STRUCT_SIZE, at, struct_end, data.len())?;

        // Done
        Ok(
            Matrix {
                vectors: [
                    Vector3D::from_tag(data, at + VECTOR_STRUCT_SIZE * 0, struct_end, &mut struct_end)?,
                    Vector3D::from_tag(data, at + VECTOR_STRUCT_SIZE * 1, struct_end, &mut struct_end)?,
                    Vector3D::from_tag(data, at + VECTOR_STRUCT_SIZE * 2, struct_end, &mut struct_end)?
                ]
            }
        )
    }
}

macro_rules! data_pointer_into_tag_assertions {
    ($at:tt, $struct_end:tt, $data:tt, $sizeof:expr) => {
        // catch programming errors
        debug_assert!(fits($sizeof, $at, $struct_end, $data.len()).is_ok());
        debug_assert_eq!($data[$at..$at+$sizeof], [0u8; $sizeof], get_compiled_string!("engine.h1.types.serialize.error_data_not_zeroed_out"), $at);
    }
}

macro_rules! data_pointer_from_tag_assertions {
    ($at:tt, $struct_end:tt, $data:tt, $sizeof:expr, $cursor:tt) => {
        // Our cursor needs to point outside of the struct. If not, that's a programmer error.
        debug_assert!(*$cursor >= $struct_end, get_compiled_string!("engine.h1.types.serialize.error_data_cursor_inside_struct"));

        // Does the base struct fit?
        fits($sizeof, $at, $struct_end, $data.len())?;
    }
}


const DATA_STRUCT_SIZE: usize = sizeof!(u32) * 5;

impl TagSerialize for Data {
    fn tag_size() -> usize {
        DATA_STRUCT_SIZE
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize)  -> ErrorMessageResult<()> {
        data_pointer_into_tag_assertions!(at, struct_end, data, DATA_STRUCT_SIZE);

        // internally this is stored as a 32-bit signed integer
        let size = self.len();
        let limit = crate::engines::h1::MAX_ARRAY_LENGTH;
        if size > limit {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.serialize.error_byte_array_limit_exceeded"), size=size, limit=limit)));
        }

        // append the data and write the length
        data.extend(self);
        (self.len() as u32).into_tag(data, at + 0x0, struct_end)
    }
    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<Self> {
        data_pointer_from_tag_assertions!(at, struct_end, data, DATA_STRUCT_SIZE, cursor);

        // Does this array fit?
        let length = u32::from_tag(data, at + 0x0, struct_end, cursor)? as usize;

        fits_extra_data(length, *cursor, data.len())?;

        // Ok, good!
        let end = *cursor + length;
        let vec = data[*cursor..end].to_owned();
        *cursor = end;
        Ok(vec)
    }
}

const TAG_REFERENCE_STRUCT_SIZE: usize = sizeof!(u32) * 4;

impl TagSerialize for TagReference {
    fn tag_size() -> usize {
        TAG_REFERENCE_STRUCT_SIZE
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {
        data_pointer_into_tag_assertions!(at, struct_end, data, TAG_REFERENCE_STRUCT_SIZE);

        // current path
        let path = self.get_path_without_extension();

        // If it's empty, we do not need to write anything.
        if self.group == TagGroup::_None && path.is_empty() {
            return Ok(())
        }

        // Write an empty ID
        (0xFFFFFFFFu32).into_tag(data, at + 0xC, struct_end)?;

        if path.is_empty() {
            (self.group.as_fourcc()).into_tag(data, at + 0x0, struct_end)
        }
        else {
            // internally this is stored as a 32-bit signed integer
            let size = path.len();
            let limit = crate::engines::h1::MAX_ARRAY_LENGTH;
            if size > limit {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.serialize.error_max_path_limit"), size=size, limit=limit)));
            }

            data.extend_from_slice(path.as_bytes());
            data.push(0); // null terminator
            (self.group.as_fourcc()).into_tag(data, at + 0x0, struct_end)?;
            (path.len() as u32).into_tag(data, at + 0x8, struct_end)
        }
    }
    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<Self> {
        data_pointer_from_tag_assertions!(at, struct_end, data, TAG_REFERENCE_STRUCT_SIZE, cursor);

        // Does this array fit?
        let length = u32::from_tag(data, at + 0x8, struct_end, cursor)?;

        if length > 0 {
            let fourcc = u32::from_tag(data, at + 0x0, struct_end, cursor)?;
            let group = match TagGroup::from_fourcc(fourcc) {
                Some(n) => n,
                None => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.serialize.error_fourcc_invalid"), fourcc=fourcc)))
            };

            let real_length = match (length as usize).checked_add(1) {
                Some(n) => n,
                None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.serialize.error_architecture_limit_exceeded")))
            };

            fits_extra_data(real_length, *cursor, data.len())?;

            // Ok, good!
            let end = *cursor + real_length;
            let bytes = &data[*cursor..end];
            *cursor = end;

            if let Ok(n) = std::str::from_utf8(&bytes[..bytes.len() - 1]) {
                TagReference::from_path_and_group(n, group)
            }
            else {
                Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.serialize.error_path_not_utf8")))
            }
        }
        else {
            let mut result = TagReference::default();
            result.group = TagGroup::from_fourcc(u32::from_tag(data, at + 0x0, struct_end, cursor)?).unwrap_or(TagGroup::_None);
            Ok(result)
        }
    }
}

const BLOCK_ARRAY_STRUCT_SIZE: usize = sizeof!(u32) * 3;

impl<T: TagBlockFn + TagSerialize + Default> TagSerialize for Reflexive<T> {
    fn tag_size() -> usize {
        BLOCK_ARRAY_STRUCT_SIZE
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {
        data_pointer_into_tag_assertions!(at, struct_end, data, BLOCK_ARRAY_STRUCT_SIZE);

        // internally this is stored as a 32-bit signed integer
        let size = self.blocks.len();
        let limit = crate::engines::h1::MAX_ARRAY_LENGTH;
        if size > limit {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.serialize.error_array_limit_exceeded"), size=size, limit=limit)));
        }

        // Get the total size
        let element_size = T::tag_size();
        let total_size = match element_size.checked_mul(size) {
            Some(n) => n,
            None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.serialize.error_architecture_limit_exceeded")))
        };

        // Get the location we will be putting our new data into
        let mut current_offset = data.len();
        let new_data_size = match current_offset.checked_add(total_size) {
            Some(n) => n,
            None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.serialize.error_architecture_limit_exceeded")))
        };
        data.resize(new_data_size, 0);

        // Go through each block and add them into the tag
        for b in &self.blocks {
            let next_offset = current_offset + element_size;
            b.into_tag(data, current_offset, next_offset)?;
            current_offset = next_offset;
        }

        // Write the new offset.
        (size as u32).into_tag(data, at + 0x0, struct_end)

    }
    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<Self> {
        data_pointer_from_tag_assertions!(at, struct_end, data, BLOCK_ARRAY_STRUCT_SIZE, cursor);

        // Does this array fit?
        let count = u32::from_tag(data, at + 0x0, struct_end, cursor)? as usize;
        let tag_size = T::tag_size();

        let total_size = match tag_size.checked_mul(count) {
            Some(n) => n,
            None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.serialize.error_architecture_limit_exceeded")))
        };

        fits_extra_data(total_size, *cursor, data.len())?;

        // Initialize our block array
        let mut block_array = Self::default();
        block_array.blocks.reserve(count);

        // set our cursor to the end of the array and record where it is now
        let mut cursor_start = *cursor;
        *cursor += total_size;

        // add each block one by one
        for _ in 0..count {
            let this_struct_end = cursor_start + tag_size;
            block_array.blocks.push(T::from_tag(data, cursor_start, this_struct_end, cursor)?);
            cursor_start = this_struct_end;
        }

        Ok(block_array)
    }
}

/// Size of the tag file header.
pub(crate) const TAG_FILE_HEADER_LEN: usize = 0x40;

impl TagSerialize for TagFileHeader {
    fn tag_size() -> usize {
        TAG_FILE_HEADER_LEN
    }

    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {
        data_pointer_into_tag_assertions!(at, struct_end, data, TAG_FILE_HEADER_LEN);

        self.old_tag_id.into_tag(data, 0x00, struct_end)?;
        self.old_tag_name.into_tag(data, 0x04, struct_end)?;
        self.tag_group.as_fourcc().into_tag(data, 0x24, struct_end)?;
        self.crc32.into_tag(data, 0x28, struct_end)?;
        self.header_length.into_tag(data, 0x2C, struct_end)?;
        self.tag_group_version.into_tag(data, 0x38, struct_end)?;
        self.something_255.into_tag(data, 0x3A, struct_end)?;
        self.blam_fourcc.into_tag(data, 0x3C, struct_end)?;

        Ok(())
    }

    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<Self> {
        data_pointer_from_tag_assertions!(at, struct_end, data, TAG_FILE_HEADER_LEN, cursor);

        let group_fourcc = u32::from_tag(data, 0x24, struct_end, cursor)?;
        let group = match TagGroup::from_fourcc(group_fourcc) {
            Some(n) => n,
            None => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.serialize.error_fourcc_invalid"), fourcc=group_fourcc)))
        };

        Ok(TagFileHeader {
            old_tag_id: u32::from_tag(data, 0x00, struct_end, cursor)?,
            old_tag_name: String32::from_tag(data, 0x04, struct_end, cursor)?,
            tag_group: group,
            crc32: u32::from_tag(data, 0x28, struct_end, cursor)?,
            header_length: u32::from_tag(data, 0x2C, struct_end, cursor)?,
            tag_group_version: u16::from_tag(data, 0x38, struct_end, cursor)?,
            something_255: u16::from_tag(data, 0x3A, struct_end, cursor)?,
            blam_fourcc: u32::from_tag(data, 0x3C, struct_end, cursor)?,
        })
    }
}

/// Functionality for parsing and making tag file data.
pub struct ParsedTagFile<T: ?Sized> {
    /// Header that was read from the tag.
    pub header: TagFileHeader,

    /// Data that was read from the tag.
    pub data: Box<T>
}

impl<T: TagSerialize> ParsedTagFile<T> {
    /// Parse the tag from the given bytes.
    pub fn from_tag(bytes: &[u8]) -> ErrorMessageResult<ParsedTagFile<T>> {
        let base_struct_size = T::tag_size();
        let mut cursor = TAG_FILE_HEADER_LEN + base_struct_size;
        let header = TagFileHeader::from_tag(bytes, 0, TAG_FILE_HEADER_LEN, &mut cursor)?;
        header.validate()?;

        let final_data = ParsedTagFile {
            header: header,
            data: Box::new(T::from_tag(bytes, TAG_FILE_HEADER_LEN, cursor, &mut cursor)?)
        };

        if cursor != bytes.len() {
            Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.serialize.error_tag_leftover_data"), read=cursor, total=bytes.len())))
        }
        else {
            Ok(final_data)
        }
    }

    /// Convert the tag data into the given bytes.
    pub fn into_tag(data: &T, tag_group: TagGroup) -> ErrorMessageResult<Vec<u8>> {
        let base_struct_size = T::tag_size();
        let struct_end = TAG_FILE_HEADER_LEN + base_struct_size;

        let mut final_data: Vec<u8> = Vec::new();
        final_data.resize(struct_end, 0);
        data.into_tag(&mut final_data, TAG_FILE_HEADER_LEN, struct_end)?;

        let crc32 = crate::crc::crc32(&final_data[TAG_FILE_HEADER_LEN..]);

        let header = TagFileHeader {
            old_tag_id: 0,
            old_tag_name: String32::default(),
            tag_group: tag_group,
            tag_group_version: TagFileHeader::version_for_group(tag_group),
            header_length: TAG_FILE_HEADER_LEN as u32,
            something_255: 255,
            blam_fourcc: BLAM_FOURCC,
            crc32
        };

        header.into_tag(&mut final_data, 0, TAG_FILE_HEADER_LEN)?;
        Ok(final_data)
    }
}

/// Functions for parsing and making tag file data for a specific tag group.
pub trait TagFileSerializeFn: Any + TagBlockFn + TagSerialize {
    /// Deserialize the data into the tag struct.
    fn from_tag_file(data: &[u8]) -> ErrorMessageResult<ParsedTagFile<Self>> where Self: Sized;

    /// Serialize the tag struct into a tag file.
    fn into_tag_file(&self) -> ErrorMessageResult<Vec<u8>>;
}

