use rat_in_a_tube::CompileTarget;

use ringhopper_proc::*;

load_target_json_def!();

/// Type of base memory address.
#[derive(Copy, Clone, PartialEq)]
pub enum BaseMemoryAddressType {
    /// The base memory address is inferred from the address of the tag array.
    ///
    /// The encapsulated value is the default value of this address.
    Inferred(u32),

    /// The base memory address is fixed to a set address and never changes.
    ///
    /// The encapsulated value is the value of this address.
    Fixed(u32)
}

impl BaseMemoryAddressType {
    /// Get the preferred base memory address.
    pub fn get_preferred_address(&self) -> u32 {
        match self {
            BaseMemoryAddressType::Fixed(n) => *n,
            BaseMemoryAddressType::Inferred(n) => *n
        }
    }

    /// Return `true` if the base memory address is inferred.
    pub fn is_inferred(&self) -> bool {
        match self {
            BaseMemoryAddressType::Fixed(_) => false,
            BaseMemoryAddressType::Inferred(_) => true
        }
    }

    /// Return `true` if the base memory address is fixed.
    pub fn is_fixed(&self) -> bool {
        match self {
            BaseMemoryAddressType::Fixed(_) => true,
            BaseMemoryAddressType::Inferred(_) => false
        }
    }
}

/// Header layout
#[derive(Copy, Clone, PartialEq)]
pub enum HeaderLayout {
    /// The header is in the standard layout.
    Standard,

    /// The header is in the Gearbox demo layout.
    GearboxDemo
}

/// Tags required for various scenario types.
#[derive(Clone, Copy)]
pub struct RequiredTags {
    /// All scenarios require these tags.
    pub all: &'static [&'static str],

    /// Singleplayer (non-demo UI) scenarios require these tags.
    pub singleplayer: &'static [&'static str],

    /// Multiplayer (non-demo UI) scenarios require these tags.
    pub multiplayer: &'static [&'static str],

    /// User interface (non-demo UI) scenarios require these tags.
    pub user_interface: &'static [&'static str],

    /// Singleplayer (demo UI only) scenarios require these tags.
    pub singleplayer_demo: &'static [&'static str],

    /// Multiplayer (demo UI only) scenarios require these tags.
    pub multiplayer_demo: &'static [&'static str],

    /// User interface (demo UI only) scenarios require these tags.
    pub user_interface_demo: &'static [&'static str]
}

/// Engine targets define limits and parameters for a release of the game.
#[derive(Copy, Clone)]
pub struct EngineTarget {
    /// Name of the engine.
    pub name: &'static str,

    /// If `true`, then if the build string does not match a known version but the cache file version does, default to this.
    pub fallback: bool,

    /// Full version string from the "version" command, if valid.
    pub version: Option<&'static str>,

    /// Shorthand of the engine if set.
    pub shorthand: Option<&'static str>,

    /// Build string of the engine, if set.
    pub build: Option<&'static str>,

    /// Version of the cache file.
    pub cache_file_version: u32,

    /// BSPs share tag space with the rest of the tag data.
    pub bsps_occupy_tag_space: bool,

    /// Maximum tag space in bytes.
    pub max_tag_space: usize,

    /// Maximum cache file size for user interface maps in bytes.
    pub max_cache_file_size_user_interface: usize,

    /// Maximum cache file size for singleplayer maps in bytes.
    pub max_cache_file_size_singleplayer: usize,

    /// Maximum cache file size for multiplayer maps in bytes.
    pub max_cache_file_size_multiplayer: usize,

    /// Base memory address.
    pub base_memory_address: BaseMemoryAddressType,

    /// Maximum number of nodes for a scenario tag.
    pub max_script_nodes: usize,

    /// Compile target for scripting.
    pub script_compile_target: CompileTarget,

    /// Tags required to build a cache file.
    pub required_tags: RequiredTags
}

impl EngineTarget {
    /// Get an engine target that matches the shorthand.
    pub fn from_shorthand(shorthand: &str) -> Option<&'static EngineTarget> {
        for i in ALL_TARGETS {
            match i.shorthand {
                Some(s) if shorthand == s => return Some(i),
                _ => continue
            }
        }
        None
    }

    /// Get an engine target that best matches the cache file version and build.
    ///
    /// The target will be the closest match, and the bool will be `true` if it is an exact match.
    ///
    /// If the bool is `false`, then it found a similar match with the same cache file version but not necessarily build as a safe fallback.
    pub fn from_cache_file_metadata(cache_file_version: u32, build: &str) -> Option<(&'static EngineTarget, bool)> {
        // Go through all targets.
        let mut fallback = None;
        for i in ALL_TARGETS {
            // If the version doesn't match, skip it.
            if i.cache_file_version != cache_file_version {
                continue;
            }

            // If this is a fallback, remember it for later.
            if i.fallback && fallback.is_none() {
                fallback = Some((i, false));
            }

            // Next, check the build. If it matches, return it.
            match i.build {
                Some(b) if b == build => return Some((i, true)),
                _ => continue
            }
        }

        // If we have a fallback, we'll return it.
        fallback
    }
}
