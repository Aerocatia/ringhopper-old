use strings::*;

mod groups;
pub use self::groups::*;

mod supergroups;
pub use self::supergroups::*;

mod model;
pub use self::model::*;

use crate::error::*;
use crate::engines::h1::tag_loading::TagSerialize;
use crate::types::Reflexive;
use crate::types::TagBlockFn;

/// Halo: CE specific [TagReference] type.
pub type TagReference = crate::types::TagReference<TagGroup>;

/// Tag ID union.
pub type TagID = u32;

/// 32-bit pointer.
pub type Pointer = u32;

/// 16-bit index.
pub type Index = Option<u16>;

/// Convenience function for attempting unwrapping a null [`Index`].
pub trait IndexFn {
    /// Attempt to unwrap the index, returning a [`usize`] if successful or an [`Err`] on failure.
    fn try_unwrap_index(&self) -> ErrorMessageResult<usize>;
}

impl IndexFn for Index {
    fn try_unwrap_index(&self) -> ErrorMessageResult<usize> {
        match *self {
            Some(n) => Ok(n as usize),
            None => Err(ErrorMessage::StaticString(get_compiled_string!("engine.types.error_null_index_unwrap")))
        }
    }
}

pub trait ReflexiveIndexFn {
    type Item;

    /// Attempt to access an element at the given index.
    ///
    /// Return [`Err`] if the index is out of bounds. Return [`None`] if the index is none. Otherwise, return a reference to the element.
    fn try_get_with_index(&self, index: Index) -> ErrorMessageResult<Option<&Self::Item>>;

    /// Attempt to access an element at the given index, requiring that the index is not null.
    ///
    /// Return [`Err`] if the index is out of bounds or null. Otherwise, return a reference to the element.
    fn try_get_with_index_nonnull(&self, index: Index) -> ErrorMessageResult<&Self::Item>;

    /// Attempt to access an element at the given index.
    ///
    /// Return [`Err`] if the index is out of bounds. Return [`None`] if the index is none. Otherwise, return a reference to the element.
    fn try_get_with_index_mut(&mut self, index: Index) -> ErrorMessageResult<Option<&mut Self::Item>>;

    /// Attempt to access an element at the given index, requiring that the index is not null.
    ///
    /// Return [`Err`] if the index is out of bounds or null. Otherwise, return a reference to the element.
    fn try_get_with_index_nonnull_mut(&mut self, index: Index) -> ErrorMessageResult<&mut Self::Item>;
}

impl<T: TagBlockFn> ReflexiveIndexFn for Reflexive<T> {
    type Item = T;

    fn try_get_with_index(&self, index: Index) -> ErrorMessageResult<Option<&T>> {
        let blocks_len = self.blocks.len();
        match index {
            Some(n) => match self.blocks.get(n as usize) {
                Some(n) => Ok(Some(n)),
                None => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_out_of_bounds_index"), item_type=std::any::type_name::<T>(), index=n, size=blocks_len)))
            },
            None => Ok(None)
        }
    }
    fn try_get_with_index_nonnull(&self, index: Index) -> ErrorMessageResult<&T> {
        let blocks_len = self.blocks.len();
        match index {
            Some(n) => match self.blocks.get(n as usize) {
                Some(n) => Ok(n),
                None => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_out_of_bounds_index"), item_type=std::any::type_name::<T>(), index=n, size=blocks_len)))
            },
            None => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_null_index"), item_type=std::any::type_name::<T>(), size=blocks_len)))
        }
    }

    fn try_get_with_index_mut(&mut self, index: Index) -> ErrorMessageResult<Option<&mut T>> {
        let blocks_len = self.blocks.len();
        match index {
            Some(n) => match self.blocks.get_mut(n as usize) {
                Some(n) => Ok(Some(n)),
                None => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_out_of_bounds_index"), item_type=std::any::type_name::<T>(), index=n, size=blocks_len)))
            },
            None => Ok(None)
        }
    }

    fn try_get_with_index_nonnull_mut(&mut self, index: Index) -> ErrorMessageResult<&mut T> {
        let blocks_len = self.blocks.len();
        match index {
            Some(n) => match self.blocks.get_mut(n as usize) {
                Some(n) => Ok(n),
                None => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_out_of_bounds_index"), item_type=std::any::type_name::<T>(), index=n, size=blocks_len)))
            },
            None => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_null_index"), item_type=std::any::type_name::<T>(), size=blocks_len)))
        }
    }
}

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
