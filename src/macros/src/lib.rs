#[cfg(target_os = "linux")]
extern crate libc;

#[cfg(target_os = "windows")]
extern crate windows;

#[macro_use]
pub mod terminal;
