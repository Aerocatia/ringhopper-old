extern crate strings;

#[cfg(target_os = "linux")]
extern crate libc;

#[cfg(target_os = "windows")]
extern crate windows;

mod error;
pub use error::*;

mod engine;
pub use engine::*;

pub mod cmd;
pub mod crc;

#[macro_use]
pub mod terminal;
