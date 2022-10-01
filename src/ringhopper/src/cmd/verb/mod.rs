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
    TagCollection,
    UICollection,
    UnicodeStrings,
    Version
}

/// Info for a verb, including the name, aliases, and description.
pub struct VerbInfo {
    /// Verb the info refers to.
    pub verb: Verb,

    name: &'static str,
    description: &'static str
}

impl VerbInfo {
    const fn new(verb: Verb, name: &'static str, description: &'static str) -> VerbInfo {
        VerbInfo { verb, name, description }
    }
}

pub(crate) static ALL_VERBS: &'static [VerbInfo] = &[
    VerbInfo::new(Verb::Archive, "archive", get_compiled_string!("verb.archive.description")),
    VerbInfo::new(Verb::Bitmap, "bitmap", get_compiled_string!("verb.bitmap.description")),
    VerbInfo::new(Verb::Bludgeon, "bludgeon", get_compiled_string!("verb.bludgeon.description")),
    VerbInfo::new(Verb::BSP, "bsp", get_compiled_string!("verb.bsp.description")),
    VerbInfo::new(Verb::Build, "build", get_compiled_string!("verb.build.description")),
    VerbInfo::new(Verb::CameraTrack, "camera-track", get_compiled_string!("verb.camera-track.description")),
    VerbInfo::new(Verb::Collision, "collision", get_compiled_string!("verb.collision.description")),
    VerbInfo::new(Verb::Compare, "compare", get_compiled_string!("verb.compare.description")),
    VerbInfo::new(Verb::Convert, "convert", get_compiled_string!("verb.convert.description")),
    VerbInfo::new(Verb::Dependency, "dependency", get_compiled_string!("verb.dependency.description")),
    VerbInfo::new(Verb::Edit, "edit", get_compiled_string!("verb.edit.description")),
    VerbInfo::new(Verb::Extract, "extract", get_compiled_string!("verb.extract.description")),
    VerbInfo::new(Verb::Font, "font", get_compiled_string!("verb.font.description")),
    VerbInfo::new(Verb::GBXModel, "gbxmodel", get_compiled_string!("verb.gbxmodel.description")),
    VerbInfo::new(Verb::HUDMessages, "hud-messages", get_compiled_string!("verb.hud-messages.description")),
    VerbInfo::new(Verb::Info, "info", get_compiled_string!("verb.info.description")),
    VerbInfo::new(Verb::Lightmap, "lightmap", get_compiled_string!("verb.lightmap.description")),
    VerbInfo::new(Verb::Model, "model", get_compiled_string!("verb.model.description")),
    VerbInfo::new(Verb::Physics, "physics", get_compiled_string!("verb.physics.description")),
    VerbInfo::new(Verb::Plate, "plate", get_compiled_string!("verb.plate.description")),
    VerbInfo::new(Verb::Recover, "recover", get_compiled_string!("verb.recover.description")),
    VerbInfo::new(Verb::Refactor, "refactor", get_compiled_string!("verb.refactor.description")),
    VerbInfo::new(Verb::Repair, "repair", get_compiled_string!("verb.repair.description")),
    VerbInfo::new(Verb::Resource, "resource", get_compiled_string!("verb.resource.description")),
    VerbInfo::new(Verb::Scan, "scan", get_compiled_string!("verb.scan.description")),
    VerbInfo::new(Verb::Script, "script", get_compiled_string!("verb.script.description")),
    VerbInfo::new(Verb::Sound, "sound", get_compiled_string!("verb.sound.description")),
    VerbInfo::new(Verb::Strip, "strip",  get_compiled_string!("verb.strip.description")),
    VerbInfo::new(Verb::Strings, "strings", get_compiled_string!("verb.strings.description")),
    VerbInfo::new(Verb::TagCollection, "tag-collection", get_compiled_string!("verb.tag-collection.description")),
    VerbInfo::new(Verb::UICollection, "ui-collection", get_compiled_string!("verb.ui-collection.description")),
    VerbInfo::new(Verb::UnicodeStrings, "unicode-strings", get_compiled_string!("verb.unicode-strings.description")),
    VerbInfo::new(Verb::Version, "version", get_compiled_string!("verb.version.description"))
];

impl Verb {
    /// Get metadata for the verb
    pub fn get_entry(&self) -> &'static VerbInfo {
        for v in ALL_VERBS {
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

    /// Get the description of the verb
    pub fn get_description(&self) -> &'static str {
        return self.get_entry().description
    }

    /// Attempt to get a verb from a given name or shorthand
    pub fn from_input(input: &str) -> Option<Self> {
        for v in ALL_VERBS {
            if input.eq_ignore_ascii_case(v.name) {
                return Some(v.verb)
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
