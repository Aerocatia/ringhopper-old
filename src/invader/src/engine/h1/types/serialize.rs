#[cfg(test)]
mod tests;

use crate::{h1::{TagGroup, TagReference, TagFileHeader}, TagGroupFn, Matrix, ErrorMessage, ErrorMessageResult};
use strings::get_compiled_string;

/// Serialization implementation for tags in tag format.
pub trait TagSerialize: Sized {
    /// Get the size of the data
    fn tag_size() -> usize;

    /// Serialize the data into tag format, returning an error on failure (except for out-of-bounds and allocation errors which will panic).
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()>;

    /// Deserialize the data from tag format, returning an error on failure (except for allocation errors which will panic).
    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<Self>;

    /// Get the size of the instance.
    fn instance_tag_size(&self) -> usize {
        Self::tag_size()
    }

    /// Deserialize into an instasnce.
    fn instance_from_tag(&self, data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<Self> {
        Self::from_tag(data, at, struct_end, cursor)
    }
}

macro_rules! sizeof {
    ($t:ty) => {
        std::mem::size_of::<$t>()
    }
}

const fn fits(size: usize, at: usize, struct_end: usize, data_size: usize) -> ErrorMessageResult<()> {
    let end = match at.checked_add(size) {
        Some(n) => n,
        None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.types.serialize.error_architecture_limit_exceeded")))
    };

    // If data is out of the struct bounds, then this is a programming error rather than bad tag data.
    debug_assert!(end <= struct_end, "Data is outside of the struct (this is a bug!)");

    // If we're outside of the data bounds, fail.
    if end > data_size {
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
serialize_for_struct!(crate::ColorAHSV, a, h, s, v);
serialize_for_struct!(crate::ColorARGB, a, r, g, b);
serialize_for_struct!(crate::ColorHSV, h, s, v);
serialize_for_struct!(crate::ColorRGB, r, g, b);
serialize_for_struct!(crate::Euler2D, y, p);
serialize_for_struct!(crate::Euler3D, y, p, r);
serialize_for_struct!(crate::Plane2D, vector, d);
serialize_for_struct!(crate::Plane3D, vector, d);
serialize_for_struct!(crate::Point2D, x, y);
serialize_for_struct!(crate::Point2DInt, x, y);
serialize_for_struct!(crate::Point3D, x, y, z);
serialize_for_struct!(crate::Quaternion, x, y, z, w);
serialize_for_struct!(crate::Rectangle, top, left, bottom, right);
serialize_for_struct!(crate::Vector2D, x, y);
serialize_for_struct!(crate::Vector3D, x, y, z);

// These convert to 32-bit integers, so we can serialize them as such
impl TagSerialize for crate::ColorARGBInt {
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

impl TagSerialize for crate::ColorRGBInt {
    fn tag_size() -> usize {
        u32::tag_size()
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, offset: usize) -> ErrorMessageResult<()> {
        crate::ColorARGBInt::from(*self).to_a8r8g8b8().into_tag(data, at, offset)
    }
    fn from_tag(data: &[u8], at: usize, offset: usize, cursor: &mut usize) -> ErrorMessageResult<Self> {
        let argb = crate::ColorARGBInt::from_a8r8g8b8(u32::from_tag(data, at, offset, cursor)?);
        Ok(Self { r: argb.r, g: argb.g, b: argb.b })
    }
}

// This is a simple 32 byte array
impl TagSerialize for crate::String32 {
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

impl TagSerialize for crate::Matrix {
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
                    crate::Vector3D::from_tag(data, at + VECTOR_STRUCT_SIZE * 0, struct_end, &mut struct_end)?,
                    crate::Vector3D::from_tag(data, at + VECTOR_STRUCT_SIZE * 1, struct_end, &mut struct_end)?,
                    crate::Vector3D::from_tag(data, at + VECTOR_STRUCT_SIZE * 2, struct_end, &mut struct_end)?
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

impl TagSerialize for crate::Data {
    fn tag_size() -> usize {
        DATA_STRUCT_SIZE
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize)  -> ErrorMessageResult<()> {
        data_pointer_into_tag_assertions!(at, struct_end, data, DATA_STRUCT_SIZE);

        // internally this is stored as a 32-bit signed integer
        let size = self.len();
        let limit = crate::h1::MAX_ARRAY_LENGTH;
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
        fits(length, *cursor, data.len(), data.len())?;

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

        if path.is_empty() {
            (self.group.to_fourcc()).into_tag(data, at + 0x0, struct_end)
        }
        else {
            // internally this is stored as a 32-bit signed integer
            let size = path.len();
            let limit = crate::h1::MAX_ARRAY_LENGTH;
            if size > limit {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.serialize.error_max_path_limit"), size=size, limit=limit)));
            }

            data.extend_from_slice(path.as_bytes());
            data.push(0); // null terminator
            (self.group.to_fourcc()).into_tag(data, at + 0x0, struct_end)?;
            (path.len() as u32).into_tag(data, at + 0xC, struct_end)
        }
    }
    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<Self> {
        data_pointer_from_tag_assertions!(at, struct_end, data, TAG_REFERENCE_STRUCT_SIZE, cursor);

        // Does this array fit?
        let length = u32::from_tag(data, at + 0xC, struct_end, cursor)?;

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

            fits(real_length, *cursor, data.len(), data.len())?;

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
            result.group = TagGroup::from_fourcc(u32::from_tag(data, at + 0x0, struct_end, cursor)?).unwrap_or(TagGroup::None);
            Ok(result)
        }
    }
}

const BLOCK_ARRAY_STRUCT_SIZE: usize = sizeof!(u32) * 3;

impl<T: crate::TagBlockFn + TagSerialize + Default> TagSerialize for crate::BlockArray<T> {
    fn tag_size() -> usize {
        BLOCK_ARRAY_STRUCT_SIZE
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {
        data_pointer_into_tag_assertions!(at, struct_end, data, BLOCK_ARRAY_STRUCT_SIZE);

        // internally this is stored as a 32-bit signed integer
        let size = self.blocks.len();
        let limit = crate::h1::MAX_ARRAY_LENGTH;
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
        fits(total_size, *cursor, data.len(), data.len())?;

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
const TAG_FILE_HEADER_LEN: usize = 0x40;

impl TagSerialize for TagFileHeader {
    fn tag_size() -> usize {
        TAG_FILE_HEADER_LEN
    }

    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {
        data_pointer_into_tag_assertions!(at, struct_end, data, TAG_FILE_HEADER_LEN);

        self.old_tag_id.into_tag(data, 0x00, struct_end)?;
        self.old_tag_name.into_tag(data, 0x04, struct_end)?;
        self.tag_group.to_fourcc().into_tag(data, 0x24, struct_end)?;
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
            old_tag_name: crate::String32::from_tag(data, 0x04, struct_end, cursor)?,
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
pub struct ParsedTagFile<T> {
    /// Header that was read from the tag.
    pub header: TagFileHeader,

    /// Data that was read from the tag.
    pub data: T
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
            data: T::from_tag(bytes, TAG_FILE_HEADER_LEN, cursor, &mut cursor)?
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
            old_tag_name: crate::String32::default(),
            tag_group: tag_group,
            tag_group_version: TagFileHeader::version_for_group(tag_group),
            header_length: TAG_FILE_HEADER_LEN as u32,
            something_255: 255,
            blam_fourcc: super::BLAM_FOURCC,
            crc32
        };

        header.into_tag(&mut final_data, 0, TAG_FILE_HEADER_LEN)?;
        Ok(final_data)
    }
}
