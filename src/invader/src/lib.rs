extern crate strings;
extern crate invader_macros;

#[cfg(target_os = "windows")]
extern crate windows;

pub mod error;
pub mod cmd;
pub mod crc;
pub mod types;

/// The current Invader version, including a git commit count/hash if available.
pub const INVADER_VERSION: &'static str = env!("invader_version");

pub mod engines;

pub mod file;
