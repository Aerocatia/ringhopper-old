use h1::TagSerialize;
use ErrorMessage;
use TagBlockFn;
use crate::BlockArray;

/// Unicode string list struct.
#[derive(PartialEq, Default)]
pub struct UnicodeStringList {
    /// List of string blocks.
    pub strings: BlockArray<UnicodeStringListString>
}

/// Unicode string list string.
#[derive(Default, PartialEq)]
pub struct UnicodeStringListString {
    /// UTF-16 data in little endian byte format.
    pub string_data: Vec<u8>
}

impl TagBlockFn for UnicodeStringListString {
    fn field_count(&self) -> usize { todo!() }
    fn field_at_index(&self, _: usize) -> crate::TagField { todo!() }
    fn field_at_index_mut(&mut self, _: usize) -> crate::TagField { todo!() }
}

impl TagSerialize for UnicodeStringList {
    fn tag_size() -> usize {
        BlockArray::<UnicodeStringListString>::tag_size()
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> Result<(), ErrorMessage> {
        self.strings.into_tag(data, at, struct_end)
    }
    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> Result<Self, ErrorMessage> {
        Ok(UnicodeStringList { strings: BlockArray::<UnicodeStringListString>::from_tag(data, at, struct_end, cursor)? })
    }
}

impl TagSerialize for UnicodeStringListString {
    fn tag_size() -> usize {
        Vec::<u8>::tag_size()
    }
    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> Result<(), ErrorMessage> {
        self.string_data.into_tag(data, at, struct_end)
    }
    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> Result<Self, ErrorMessage> {
        Ok(UnicodeStringListString { string_data: Vec::<u8>::from_tag(data, at, struct_end, cursor)? })
    }
}
