mod groups;
pub use self::groups::*;

mod supergroups;
pub use self::supergroups::*;

use crate::error::*;
use crate::engines::h1::tag_loading::TagSerialize;

/// Halo: CE specific [TagReference] type.
pub type TagReference = crate::types::TagReference<TagGroup>;

/// Tag ID union.
pub type TagID = u32;

/// 32-bit pointer.
pub type Pointer = u32;

/// 16-bit index.
pub type Index = u16;

/// Refers to a value stored in a script node.
///
/// Accessing this directly is technically unsafe as there is no way to know what is stored here. If its respective
/// node is non-primitive or you access the wrong type, you may get a garbage value.
///
/// Also, using [`ScenarioScriptNodeValue::eq`] on a `ScenarioScriptNodeValue` instance will return `false` if the data
/// outside of the actual value is different as this only does a binary comparison.
#[derive(Clone, Copy)]
pub union ScenarioScriptNodeValue {
    pub bool_int: i8,
    pub short_int: i16,
    pub long_int: i32,
    pub real: f32,
    pub id: TagID,
    pub unsigned_long_int: u32
}
impl Default for ScenarioScriptNodeValue {
    fn default() -> Self {
        Self { unsigned_long_int: u32::default() }
    }
}
impl PartialEq for ScenarioScriptNodeValue {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.unsigned_long_int == other.unsigned_long_int }
    }
}
impl TagSerialize for ScenarioScriptNodeValue {
    fn tag_size() -> usize {
        4
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {
        unsafe { self.unsigned_long_int.into_tag(data, at, struct_end) }
    }
    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<ScenarioScriptNodeValue> {
        Ok(ScenarioScriptNodeValue { unsigned_long_int: u32::from_tag(data, at, struct_end, cursor)? })
    }
}
