use std::any::Any;
use std::ops::{Index, IndexMut};

#[cfg(test)]
mod tests;

/// General interface for tag group parsing.
pub trait TagGroupFn where Self: Sized {
    /// Get a tag group from the FourCC signature or `None` if the FourCC is unrecognized.
    fn from_fourcc(fourcc: u32) -> Option<Self>;

    /// Get the name of the tag group used in tag extensions.
    fn to_fourcc(&self) -> u32;

    /// Get the name of the tag group used in tag extensions.
    fn as_str(&self) -> &'static str;

    /// Get a tag group from a string or `None` if the string is unrecognized.
    fn from_str(str: &str) -> Option<Self>;

    /// Get the `None` value of this tag group.
    fn none() -> Self;
}

/// Reference to a value in a tag.
pub struct FieldReference<T> {
    /// Field being accessed.
    pub field: T,

    /// Name of the field.
    pub name: &'static str,

    /// Name of the field.
    pub comment: &'static str
}

/// General interface for accessing the tag structure via reflection.
pub trait TagBlockFn: Any {
    /// Get the number of fields.
    fn field_count(&self) -> usize;

    /// Get the data of the field at the given index. Panics if it is out of bounds.
    fn field_at_index(&self, index: usize) -> FieldReference<&dyn Any>;

    /// Get the mutable data of the field at the given index. Panics if it is out of bounds.
    fn field_at_index_mut(&mut self, index: usize) -> FieldReference<&mut dyn Any>;

    /// Get the array at the field at the given index. Panics if it is out of bounds or is not an array.
    fn array_at_index(&self, index: usize) -> &dyn ArrayFn;

    /// Get the mutable array at the field at the given index. Panics if it is out of bounds or is not an array.
    fn array_at_index_mut(&mut self, index: usize) -> &mut dyn ArrayFn;

    /// Return `true` if the field at the given index is an array. Panics if it is out of bounds.
    fn field_at_index_is_array(&self, index: usize) -> bool;
}

/// Array which can hold multiple blocks.
#[derive(Default)]
pub struct Array<T: TagBlockFn> {
    pub blocks: Vec<T>
}

/// Interface for accessing blocks from an [Array] type.
pub trait ArrayFn {
    /// Get the length of the array.
    fn len(&self) -> usize;

    /// Get the block at the index or panic if out of bounds.
    fn block_at_index(&self, index: usize) -> &dyn TagBlockFn;

    /// Get the mutable block at the index or panic if out of bounds.
    fn block_at_index_mut(&mut self, index: usize) -> &mut dyn TagBlockFn;
}

impl<T: TagBlockFn> ArrayFn for Array<T> {
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

impl<T: TagBlockFn> Index<usize> for Array<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks[index]
    }
}

impl<T: TagBlockFn> IndexMut<usize> for Array<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.blocks[index]
    }
}

impl Index<usize> for &dyn ArrayFn {
    type Output = dyn TagBlockFn;
    fn index(&self, index: usize) -> &Self::Output {
        self.block_at_index(index)
    }
}

impl Index<usize> for &mut dyn ArrayFn {
    type Output = dyn TagBlockFn;
    fn index(&self, index: usize) -> &Self::Output {
        self.block_at_index(index)
    }
}

impl IndexMut<usize> for &mut dyn ArrayFn {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.block_at_index_mut(index)
    }
}
