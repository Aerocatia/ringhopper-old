use strings::get_compiled_string;

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
    VerbInfo::new(Verb::Archive, "archive", &["ar"], get_compiled_string!("verb.archive.description")),
    VerbInfo::new(Verb::Bitmap, "bitmap", &["bmp"], get_compiled_string!("verb.bitmap.description")),
    VerbInfo::new(Verb::Bludgeon, "bludgeon", &["bl"], get_compiled_string!("verb.bludgeon.description")),
    VerbInfo::new(Verb::BSP, "bsp", &["bs"], get_compiled_string!("verb.bsp.description")),
    VerbInfo::new(Verb::Build, "build", &["b", "build-cache-file"], get_compiled_string!("verb.build.description")),
    VerbInfo::new(Verb::CameraTrack, "camera-track", &["cam"], get_compiled_string!("verb.camera-track.description")),
    VerbInfo::new(Verb::Collection, "collection", &["tc"], get_compiled_string!("verb.collection.description")),
    VerbInfo::new(Verb::Collision, "collision", &["col"], get_compiled_string!("verb.collision.description")),
    VerbInfo::new(Verb::Compare, "compare", &["cmp"], get_compiled_string!("verb.compare.description")),
    VerbInfo::new(Verb::Convert, "convert", &["cvt"], get_compiled_string!("verb.convert.description")),
    VerbInfo::new(Verb::Dependency, "dependency", &["dep"], get_compiled_string!("verb.dependency.description")),
    VerbInfo::new(Verb::Edit, "edit", &["e"], get_compiled_string!("verb.edit.description")),
    VerbInfo::new(Verb::Extract, "extract", &["x"], get_compiled_string!("verb.extract.description")),
    VerbInfo::new(Verb::Font, "font", &["fnt"], get_compiled_string!("verb.font.description")),
    VerbInfo::new(Verb::GBXModel, "gbxmodel", &["gbx"], get_compiled_string!("verb.gbxmodel.description")),
    VerbInfo::new(Verb::HUDMessages, "hud-messages", &["hm"], get_compiled_string!("verb.hud-messages.description")),
    VerbInfo::new(Verb::Info, "info", &["i"], get_compiled_string!("verb.info.description")),
    VerbInfo::new(Verb::Lightmap, "lightmap", &["lm"], get_compiled_string!("verb.lightmap.description")),
    VerbInfo::new(Verb::Model, "model", &["mdl"], get_compiled_string!("verb.model.description")),
    VerbInfo::new(Verb::Physics, "physics", &["ph"], get_compiled_string!("verb.physics.description")),
    VerbInfo::new(Verb::Plate, "plate", &["pl"], get_compiled_string!("verb.plate.description")),
    VerbInfo::new(Verb::Recover, "recover", &["rec"], get_compiled_string!("verb.recover.description")),
    VerbInfo::new(Verb::Refactor, "refactor", &["ref"], get_compiled_string!("verb.refactor.description")),
    VerbInfo::new(Verb::Repair, "repair", &["r"], get_compiled_string!("verb.repair.description")),
    VerbInfo::new(Verb::Resource, "resource", &["res"], get_compiled_string!("verb.resource.description")),
    VerbInfo::new(Verb::Scan, "scan", &["scn"], get_compiled_string!("verb.scan.description")),
    VerbInfo::new(Verb::Script, "script", &["s"], get_compiled_string!("verb.script.description")),
    VerbInfo::new(Verb::Sound, "sound", &["snd"], get_compiled_string!("verb.sound.description")),
    VerbInfo::new(Verb::Strip, "strip",  &["st"], get_compiled_string!("verb.strip.description")),
    VerbInfo::new(Verb::Strings, "strings", &["str"], get_compiled_string!("verb.strings.description")),
    VerbInfo::new(Verb::UnicodeStrings, "unicode-strings", &["uni"], get_compiled_string!("verb.unicode-strings.description")),
    VerbInfo::new(Verb::Version, "version", &["ver"], get_compiled_string!("verb.version.description"))
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
