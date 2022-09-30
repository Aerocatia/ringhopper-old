//! Engine specific functionality, including types and functions.

pub mod h1;
use std::process::ExitCode;
use crate::cmd::Verb;
use crate::error::ErrorMessageResult;

/// Execute the verb with the given arguments, returning the exit code.
pub type VerbFn = fn (verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode>;

/// Engine modules define engines supported by the driver.
#[allow(unused)]
pub trait EngineModuleFn {
    /// Get the function for the verb if it exists.
    fn get_verb_function(&self, verb: Verb) -> Option<VerbFn>;

    /// Get the version of the engine module.
    fn get_version(&self) -> &'static str;
}

/// Message types for [`HUDMessageTextElement`](crate::engines::h1::definitions::HUDMessageTextElement) elements.
pub const HUD_MESSAGE_ELEMENT_TYPES: &'static [&'static str] = &[
    "a-button",
    "b-button",
    "x-button",
    "y-button",
    "black-button",
    "white-button",
    "left-trigger",
    "right-trigger",
    "dpad-up",
    "dpad-down",
    "dpad-left",
    "dpad-right",
    "start-button",
    "back-button",
    "left-thumb",
    "right-thumb",
    "left-stick",
    "right-stick",
    "action",
    "throw-grenade",
    "primary-trigger",
    "integrated-light",
    "jump",
    "use-equipment",
    "rotate-weapons",
    "rotate-grenades",
    "zoom",
    "crouch",
    "accept",
    "back",
    "move",
    "look",
    "custom-1",
    "custom-2",
    "custom-3",
    "custom-4",
    "custom-5",
    "custom-6",
    "custom-7",
    "custom-8"
];
