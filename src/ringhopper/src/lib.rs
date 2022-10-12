extern crate strings;
extern crate macros;
extern crate riat;

#[cfg(target_os = "windows")]
extern crate windows;

pub mod error;
pub mod cmd;
pub mod crc;
pub mod types;
pub mod bitmap;

/// The current Ringhopper version, including a git commit count/hash if available.
pub const RINGHOPPER_VERSION: &'static str = env!("ringhopper_version");

pub mod engines;

pub mod file;
