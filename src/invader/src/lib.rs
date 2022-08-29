extern crate strings;

#[cfg(target_os = "linux")]
extern crate libc;

#[cfg(target_os = "windows")]
extern crate windows;

mod error;
pub use error::*;

pub mod cmd;
pub mod crc;
pub mod types;

#[macro_use]
pub mod terminal;

/// The current Invader version, including a git commit count/hash if available.
pub const INVADER_VERSION: &'static str = env!("invader_version");

pub mod engines;
