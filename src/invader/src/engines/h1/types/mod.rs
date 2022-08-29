mod groups;
pub use self::groups::*;

/// Halo: CE specific [TagReference] type.
pub type TagReference = crate::types::TagReference<TagGroup>;
