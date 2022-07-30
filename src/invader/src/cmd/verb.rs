/// Verbs define different actions that can be called by the driver for an engine module.
#[derive(Copy, Clone, PartialEq)]
pub enum Verb {
    Archive,
    Bitmap,
    Bludgeon,
    BSP,
    Build,
    CameraTrack,
    Collection,
    Collision,
    Compare,
    Convert,
    Dependency,
    Edit,
    Extract,
    Font,
    GBXModel,
    HUDMessages,
    Info,
    Lightmap,
    Model,
    Physics,
    Plate,
    Recover,
    Repair,
    Refactor,
    Resource,
    Scan,
    Script,
    Sound,
    Strip,
    Strings,
    UnicodeStrings,
    Version
}

/// Info for a verb, including the name, aliases, and description.
pub struct VerbInfo {
    /// Verb the info refers to.
    pub verb: Verb,

    name: &'static str,
    aliases: &'static [&'static str],
    description: &'static str
}

impl VerbInfo {
    const fn new(verb: Verb, name: &'static str, aliases: &'static [&'static str], description: &'static str) -> VerbInfo {
        VerbInfo { verb, name, aliases, description }
    }
}

pub(crate) static ALL_VERBS: [VerbInfo; 32] = [
    VerbInfo::new(Verb::Archive, "archive", &["ar"], "Recursively archive tags."),
    VerbInfo::new(Verb::Bitmap, "bitmap", &["bmp"], "Generate bitmap tags."),
    VerbInfo::new(Verb::Bludgeon, "bludgeon", &["bl"], "Repair tags."),
    VerbInfo::new(Verb::BSP, "bsp", &["bs"], "Generate structure_structure_bsp tags."),
    VerbInfo::new(Verb::Build, "build", &["b", "build-cache-file"], "Generate cache files."),
    VerbInfo::new(Verb::CameraTrack, "camera-track", &["cam"], "Generate camera_track tags."),
    VerbInfo::new(Verb::Collection, "collection", &["tc"], "Generate tag_collection tags."),
    VerbInfo::new(Verb::Collision, "collision", &["col"], "Generate model_collision_geometry tags."),
    VerbInfo::new(Verb::Compare, "compare", &["cmp"], "Compare sets of tags."),
    VerbInfo::new(Verb::Convert, "convert", &["cvt"], "Convert tags to other groups."),
    VerbInfo::new(Verb::Dependency, "dependency", &["dep"], "Get dependencies for tags."),
    VerbInfo::new(Verb::Edit, "edit", &["e"], "Edit tags."),
    VerbInfo::new(Verb::Extract, "extract", &["x"], "Extract tags from cache files."),
    VerbInfo::new(Verb::Font, "font", &["fnt"], "Generate font tags."),
    VerbInfo::new(Verb::GBXModel, "gbxmodel", &["gbx"], "Generate gbxmodel tags."),
    VerbInfo::new(Verb::HUDMessages, "hud-messages", &["hm"], "Generate hud_message_text tags."),
    VerbInfo::new(Verb::Info, "info", &["i"], "Get information for cache files."),
    VerbInfo::new(Verb::Lightmap, "lightmap", &["lm"], "Generate lightmaps."),
    VerbInfo::new(Verb::Model, "model", &["mdl"], "Generate model tags."),
    VerbInfo::new(Verb::Physics, "physics", &["ph"], "Generate physics tags."),
    VerbInfo::new(Verb::Plate, "plate", &["pl"], "Generate color plates."),
    VerbInfo::new(Verb::Recover, "recover", &["rec"], "Recover source data from tags."),
    VerbInfo::new(Verb::Refactor, "refactor", &["ref"], "Move tags while preserving references."),
    VerbInfo::new(Verb::Repair, "repair", &["r"], "Repair cache files."),
    VerbInfo::new(Verb::Resource, "resource", &["res"], "Generate resource maps."),
    VerbInfo::new(Verb::Scan, "scan", &["scn"], "Scan cache files for unknown data."),
    VerbInfo::new(Verb::Script, "script", &["s"], "Compile scripts for scenario tags."),
    VerbInfo::new(Verb::Sound, "sound", &["snd"], "Generate sound tags."),
    VerbInfo::new(Verb::Strip, "strip",  &["st"], "Remove unused data from tags."),
    VerbInfo::new(Verb::Strings, "strings", &["str"], "Generate string_list tags."),
    VerbInfo::new(Verb::UnicodeStrings, "unicode-strings", &["uni"], "Generate unicode_string_list tags."),
    VerbInfo::new(Verb::Version, "version", &["ver"], "Get invader's version.")
];

impl Verb {
    /// Get metadata for the verb
    pub fn get_entry(&self) -> &'static VerbInfo {
        for v in &ALL_VERBS {
            if v.verb == *self {
                return v
            }
        }
        unreachable!()
    }

    /// Get the name of the verb
    pub fn get_name(&self) -> &'static str {
        return self.get_entry().name
    }

    /// Get the shorthand of the verb
    pub fn get_shorthand(&self) -> &'static str {
        let aliases = self.get_entry().aliases;

        if aliases.is_empty() {
            ""
        }
        else {
            self.get_entry().aliases[0]
        }
    }

    /// Get the description of the verb
    pub fn get_description(&self) -> &'static str {
        return self.get_entry().description
    }

    /// Attempt to get a verb from a given name or shorthand
    pub fn from_input(input: &str) -> Option<Self> {
        for v in &ALL_VERBS {
            if input.eq_ignore_ascii_case(v.name) {
                return Some(v.verb)
            }
            for alt in v.aliases {
                if input.eq_ignore_ascii_case(alt) {
                    return Some(v.verb)
                }
            }
        }
        None
    }
}

impl std::fmt::Display for Verb {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(self.get_name()) 
    }
}
