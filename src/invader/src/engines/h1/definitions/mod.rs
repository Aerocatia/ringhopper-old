extern crate h1_code_generator;
use self::h1_code_generator::load_json_def;
use crate::engines::h1::TagSerialize;

use crate::error::*;
use crate::types::*;
use crate::types::tag::{TagBlockFn, TagField};

load_json_def!("unicode_string_list.json");
