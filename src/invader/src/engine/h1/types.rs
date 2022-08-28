mod groups;
pub use self::groups::*;

/// Halo: CE specific [TagReference] type.
pub type TagReference = crate::types::TagReference<TagGroup>;

mod serialize;
pub use self::serialize::{TagSerialize, ParsedTagFile};

mod tag;
pub use self::tag::*;
