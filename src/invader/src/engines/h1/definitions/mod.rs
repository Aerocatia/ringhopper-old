//! Halo: Combat Evolved tag definitions.

extern crate h1_code_generator;
use self::h1_code_generator::load_json_def;
use crate::engines::h1::{TagSerialize, TagSerializeSwapped, TagFileSerializeFn, TagReference, ScenarioScriptNodeValue, Index, TagID, Pointer, TAG_FILE_HEADER_LEN, TagGroup, ParsedTagFile, TagFileHeader};

use crate::error::*;
use crate::types::*;
use crate::types::tag::{TagBlockFn, TagField, TagGroupFn};

use strings::*;

load_json_def!();
