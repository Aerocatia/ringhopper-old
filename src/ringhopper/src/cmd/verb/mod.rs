use strings::get_compiled_string;

/// Verbs define different actions that can be called by the driver for an engine module.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
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
    ListEngines,
    Model,
    Physics,
    Plate,
    Recover,
    Refactor,
    Repair,
    Resource,
    Scan,
    Script,
    Sound,
    Strings,
    Strip,
    TagCollection,
    UICollection,
    UnicodeStrings
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
    VerbInfo::new(Verb::ListEngines, "list-engines", get_compiled_string!("verb.list-engines.description")),
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
    VerbInfo::new(Verb::Strings, "strings", get_compiled_string!("verb.strings.description")),
    VerbInfo::new(Verb::Strip, "strip",  get_compiled_string!("verb.strip.description")),
    VerbInfo::new(Verb::TagCollection, "tag-collection", get_compiled_string!("verb.tag-collection.description")),
    VerbInfo::new(Verb::UICollection, "ui-collection", get_compiled_string!("verb.ui-collection.description")),
    VerbInfo::new(Verb::UnicodeStrings, "unicode-strings", get_compiled_string!("verb.unicode-strings.description"))
];

impl Verb {
    /// Get metadata for the verb
    pub fn get_info(&self) -> &'static VerbInfo {
        &ALL_VERBS[*self as usize]
    }

    /// Get the name of the verb
    pub fn get_name(&self) -> &'static str {
        return self.get_info().name
    }

    /// Get the description of the verb
    pub fn get_description(&self) -> &'static str {
        return self.get_info().description
    }

    /// Attempt to get a verb from a given name
    pub fn from_input(input: &str) -> Option<Self> {
        ALL_VERBS.binary_search_by(|f| f.name.cmp(input))
                 .ok()
                 .map(|index| ALL_VERBS[index].verb)
    }
}

impl std::fmt::Display for Verb {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(self.get_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbs_array_valid() {
        // Make sure it's sorted
        for i in 0..ALL_VERBS.len()-1 {
            assert!(ALL_VERBS[i].name < ALL_VERBS[i + 1].name, "{} < {} name failed", ALL_VERBS[i].verb, ALL_VERBS[i + 1].verb);
            assert!(ALL_VERBS[i].verb < ALL_VERBS[i + 1].verb, "{} < {} verb failed", ALL_VERBS[i].verb, ALL_VERBS[i + 1].verb);
        }

        // Make sure that using get_info() on a VerbInfo's verb returns the same exact VerbInfo
        for i in ALL_VERBS {
            assert_eq!(i as *const VerbInfo, i.verb.get_info() as *const VerbInfo, "VerbInfo {} address does not correspond to itself", i.verb);
        }

        // Make sure Verb is the same too
        for i in 0..ALL_VERBS.len() {
            let verb = &ALL_VERBS[i];
            assert_eq!(i, verb.verb as usize, "VerbInfo {} verb enum does not correspond to itself", verb.name);
        }
    }
}
