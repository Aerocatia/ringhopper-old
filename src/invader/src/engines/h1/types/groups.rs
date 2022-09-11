use crate::types::FourCC;
use crate::types::tag::TagGroupFn;
use std::fmt;

/// Tag groups define types of tags.
#[derive(Copy, Clone, PartialEq, Debug, PartialOrd, Ord, Eq)]
pub enum TagGroup {
    /// Corresponds to the [`Actor`](crate::engines::h1::definitions::Actor) struct.
    Actor,
    /// Corresponds to the [`ActorVariant`](crate::engines::h1::definitions::ActorVariant) struct.
    ActorVariant,
    /// Corresponds to the [`Antenna`](crate::engines::h1::definitions::Antenna) struct.
    Antenna,
    /// Corresponds to the [`Biped`](crate::engines::h1::definitions::Biped) struct.
    Biped,
    /// Corresponds to the [`Bitmap`](crate::engines::h1::definitions::Bitmap) struct.
    Bitmap,
    /// Corresponds to the [`CameraTrack`](crate::engines::h1::definitions::CameraTrack) struct.
    CameraTrack,
    /// Corresponds to the [`ColorTable`](crate::engines::h1::definitions::ColorTable) struct.
    ColorTable,
    /// Corresponds to the [`ContinuousDamageEffect`](crate::engines::h1::definitions::ContinuousDamageEffect) struct.
    ContinuousDamageEffect,
    /// Corresponds to the [`Contrail`](crate::engines::h1::definitions::Contrail) struct.
    Contrail,
    /// Corresponds to the [`DamageEffect`](crate::engines::h1::definitions::DamageEffect) struct.
    DamageEffect,
    /// Corresponds to the [`Decal`](crate::engines::h1::definitions::Decal) struct.
    Decal,
    /// Corresponds to the [`DetailObjectCollection`](crate::engines::h1::definitions::DetailObjectCollection) struct.
    DetailObjectCollection,
    /// Corresponds to the [`Device`](crate::engines::h1::definitions::Device) struct.
    Device,
    /// Corresponds to the [`DeviceControl`](crate::engines::h1::definitions::DeviceControl) struct.
    DeviceControl,
    /// Corresponds to the [`DeviceLightFixture`](crate::engines::h1::definitions::DeviceLightFixture) struct.
    DeviceLightFixture,
    /// Corresponds to the [`DeviceMachine`](crate::engines::h1::definitions::DeviceMachine) struct.
    DeviceMachine,
    /// Corresponds to the [`Dialogue`](crate::engines::h1::definitions::Dialogue) struct.
    Dialogue,
    /// Corresponds to the [`Effect`](crate::engines::h1::definitions::Effect) struct.
    Effect,
    /// Corresponds to the [`Equipment`](crate::engines::h1::definitions::Equipment) struct.
    Equipment,
    /// Corresponds to the [`Flag`](crate::engines::h1::definitions::Flag) struct.
    Flag,
    /// Corresponds to the [`Fog`](crate::engines::h1::definitions::Fog) struct.
    Fog,
    /// Corresponds to the [`Font`](crate::engines::h1::definitions::Font) struct.
    Font,
    /// Corresponds to the [`Garbage`](crate::engines::h1::definitions::Garbage) struct.
    Garbage,
    /// Corresponds to the [`GBXModel`](crate::engines::h1::definitions::GBXModel) struct.
    GBXModel,
    /// Corresponds to the [`Globals`](crate::engines::h1::definitions::Globals) struct.
    Globals,
    /// Corresponds to the [`Glow`](crate::engines::h1::definitions::Glow) struct.
    Glow,
    /// Corresponds to the [`GrenadeHUDInterface`](crate::engines::h1::definitions::GrenadeHUDInterface) struct.
    GrenadeHUDInterface,
    /// Corresponds to the [`HUDGlobals`](crate::engines::h1::definitions::HUDGlobals) struct.
    HUDGlobals,
    /// Corresponds to the [`HUDMessageText`](crate::engines::h1::definitions::HUDMessageText) struct.
    HUDMessageText,
    /// Corresponds to the [`HUDNumber`](crate::engines::h1::definitions::HUDNumber) struct.
    HUDNumber,
    /// Corresponds to the [`InputDeviceDefaults`](crate::engines::h1::definitions::InputDeviceDefaults) struct.
    InputDeviceDefaults,
    /// Corresponds to the [`Item`](crate::engines::h1::definitions::Item) struct.
    Item,
    /// Corresponds to the [`ItemCollection`](crate::engines::h1::definitions::ItemCollection) struct.
    ItemCollection,
    /// Corresponds to the [`LensFlare`](crate::engines::h1::definitions::LensFlare) struct.
    LensFlare,
    /// Corresponds to the [`Light`](crate::engines::h1::definitions::Light) struct.
    Light,
    /// Corresponds to the [`Lightning`](crate::engines::h1::definitions::Lightning) struct.
    Lightning,
    /// Corresponds to the [`LightVolume`](crate::engines::h1::definitions::LightVolume) struct.
    LightVolume,
    /// Corresponds to the [`MaterialEffects`](crate::engines::h1::definitions::MaterialEffects) struct.
    MaterialEffects,
    /// Corresponds to the [`Meter`](crate::engines::h1::definitions::Meter) struct.
    Meter,
    /// Corresponds to the [`Model`](crate::engines::h1::definitions::Model) struct.
    Model,
    /// Corresponds to the [`ModelAnimations`](crate::engines::h1::definitions::ModelAnimations) struct.
    ModelAnimations,
    /// Corresponds to the [`ModelCollisionGeometry`](crate::engines::h1::definitions::ModelCollisionGeometry) struct.
    ModelCollisionGeometry,
    /// Corresponds to the [`MultiplayerScenarioDescription`](crate::engines::h1::definitions::MultiplayerScenarioDescription) struct.
    MultiplayerScenarioDescription,
    /// Corresponds to the [`Object`](crate::engines::h1::definitions::Object) struct.
    Object,
    /// Corresponds to the [`Particle`](crate::engines::h1::definitions::Particle) struct.
    Particle,
    /// Corresponds to the [`ParticleSystem`](crate::engines::h1::definitions::ParticleSystem) struct.
    ParticleSystem,
    /// Corresponds to the [`Physics`](crate::engines::h1::definitions::Physics) struct.
    Physics,
    /// Corresponds to the [`Placeholder`](crate::engines::h1::definitions::Placeholder) struct.
    Placeholder,
    /// Corresponds to the [`PointPhysics`](crate::engines::h1::definitions::PointPhysics) struct.
    PointPhysics,
    /// Corresponds to the [`PreferencesNetworkGame`](crate::engines::h1::definitions::PreferencesNetworkGame) struct.
    PreferencesNetworkGame,
    /// Corresponds to the [`Projectile`](crate::engines::h1::definitions::Projectile) struct.
    Projectile,
    /// Corresponds to the [`Scenario`](crate::engines::h1::definitions::Scenario) struct.
    Scenario,
    /// Corresponds to the [`ScenarioStructureBSP`](crate::engines::h1::definitions::ScenarioStructureBSP) struct.
    ScenarioStructureBSP,
    /// Corresponds to the [`Scenery`](crate::engines::h1::definitions::Scenery) struct.
    Scenery,
    /// Corresponds to the [`Shader`](crate::engines::h1::definitions::Shader) struct.
    Shader,
    /// Corresponds to the [`ShaderEnvironment`](crate::engines::h1::definitions::ShaderEnvironment) struct.
    ShaderEnvironment,
    /// Corresponds to the [`ShaderModel`](crate::engines::h1::definitions::ShaderModel) struct.
    ShaderModel,
    /// Corresponds to the [`ShaderTransparentChicago`](crate::engines::h1::definitions::ShaderTransparentChicago) struct.
    ShaderTransparentChicago,
    /// Corresponds to the [`ShaderTransparentChicagoExtended`](crate::engines::h1::definitions::ShaderTransparentChicagoExtended) struct.
    ShaderTransparentChicagoExtended,
    /// Corresponds to the [`ShaderTransparentGeneric`](crate::engines::h1::definitions::ShaderTransparentGeneric) struct.
    ShaderTransparentGeneric,
    /// Corresponds to the [`ShaderTransparentGlass`](crate::engines::h1::definitions::ShaderTransparentGlass) struct.
    ShaderTransparentGlass,
    /// Corresponds to the [`ShaderTransparentMeter`](crate::engines::h1::definitions::ShaderTransparentMeter) struct.
    ShaderTransparentMeter,
    /// Corresponds to the [`ShaderTransparentPlasma`](crate::engines::h1::definitions::ShaderTransparentPlasma) struct.
    ShaderTransparentPlasma,
    /// Corresponds to the [`ShaderTransparentWater`](crate::engines::h1::definitions::ShaderTransparentWater) struct.
    ShaderTransparentWater,
    /// Corresponds to the [`Sky`](crate::engines::h1::definitions::Sky) struct.
    Sky,
    /// Corresponds to the [`Sound`](crate::engines::h1::definitions::Sound) struct.
    Sound,
    /// Corresponds to the [`SoundEnvironment`](crate::engines::h1::definitions::SoundEnvironment) struct.
    SoundEnvironment,
    /// Corresponds to the [`SoundLooping`](crate::engines::h1::definitions::SoundLooping) struct.
    SoundLooping,
    /// Corresponds to the [`SoundScenery`](crate::engines::h1::definitions::SoundScenery) struct.
    SoundScenery,
    /// Corresponds to the the removed Spheroid tag.
    Spheroid,
    /// Corresponds to the [`StringList`](crate::engines::h1::definitions::StringList) struct.
    StringList,
    /// Corresponds to the [`TagCollection`](crate::engines::h1::definitions::TagCollection) struct.
    TagCollection,
    /// Corresponds to the [`UIWidgetCollection`](crate::engines::h1::definitions::UIWidgetCollection) struct.
    UIWidgetCollection,
    /// Corresponds to the [`UIWidgetDefinition`](crate::engines::h1::definitions::UIWidgetDefinition) struct.
    UIWidgetDefinition,
    /// Corresponds to the [`UnicodeStringList`](crate::engines::h1::definitions::UnicodeStringList) struct.
    UnicodeStringList,
    /// Corresponds to the [`Unit`](crate::engines::h1::definitions::Unit) struct.
    Unit,
    /// Corresponds to the [`UnitHUDInterface`](crate::engines::h1::definitions::UnitHUDInterface) struct.
    UnitHUDInterface,
    /// Corresponds to the [`Vehicle`](crate::engines::h1::definitions::Vehicle) struct.
    Vehicle,
    /// Corresponds to the [`VirtualKeyboard`](crate::engines::h1::definitions::VirtualKeyboard) struct.
    VirtualKeyboard,
    /// Corresponds to the [`Weapon`](crate::engines::h1::definitions::Weapon) struct.
    Weapon,
    /// Corresponds to the [`WeaponHUDInterface`](crate::engines::h1::definitions::WeaponHUDInterface) struct.
    WeaponHUDInterface,
    /// Corresponds to the [`WeatherParticleSystem`](crate::engines::h1::definitions::WeatherParticleSystem) struct.
    WeatherParticleSystem,
    /// Corresponds to the [`Wind`](crate::engines::h1::definitions::Wind) struct.
    Wind,

    /// Special group for when there is an absence of a group.
    _None,
}

impl fmt::Display for TagGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(self.as_str())
    }
}

/// All tag groups for CE sorted alphabetically to allow for efficient binary searching.
const ALL_GROUPS: &'static [(&'static str, TagGroup, FourCC)] = &[
    ("actor", TagGroup::Actor, 0x61637472),
    ("actor_variant", TagGroup::ActorVariant, 0x61637476),
    ("antenna", TagGroup::Antenna, 0x616E7421),
    ("biped", TagGroup::Biped, 0x62697064),
    ("bitmap", TagGroup::Bitmap, 0x6269746D),
    ("camera_track", TagGroup::CameraTrack, 0x7472616B),
    ("color_table", TagGroup::ColorTable, 0x636F6C6F),
    ("continuous_damage_effect", TagGroup::ContinuousDamageEffect, 0x63646D67),
    ("contrail", TagGroup::Contrail, 0x636F6E74),
    ("damage_effect", TagGroup::DamageEffect, 0x6A707421),
    ("decal", TagGroup::Decal, 0x64656361),
    ("detail_object_collection", TagGroup::DetailObjectCollection, 0x646F6263),
    ("device", TagGroup::Device, 0x64657669),
    ("device_control", TagGroup::DeviceControl, 0x6374726C),
    ("device_light_fixture", TagGroup::DeviceLightFixture, 0x6C696669),
    ("device_machine", TagGroup::DeviceMachine, 0x6D616368),
    ("dialogue", TagGroup::Dialogue, 0x75646C67),
    ("effect", TagGroup::Effect, 0x65666665),
    ("equipment", TagGroup::Equipment, 0x65716970),
    ("flag", TagGroup::Flag, 0x666C6167),
    ("fog", TagGroup::Fog, 0x666F6720),
    ("font", TagGroup::Font, 0x666F6E74),
    ("garbage", TagGroup::Garbage, 0x67617262),
    ("gbxmodel", TagGroup::GBXModel, 0x6D6F6432),
    ("globals", TagGroup::Globals, 0x6D617467),
    ("glow", TagGroup::Glow, 0x676C7721),
    ("grenade_hud_interface", TagGroup::GrenadeHUDInterface, 0x67726869),
    ("hud_globals", TagGroup::HUDGlobals, 0x68756467),
    ("hud_message_text", TagGroup::HUDMessageText, 0x686D7420),
    ("hud_number", TagGroup::HUDNumber, 0x68756423),
    ("input_device_defaults", TagGroup::InputDeviceDefaults, 0x64657663),
    ("item", TagGroup::Item, 0x6974656D),
    ("item_collection", TagGroup::ItemCollection, 0x69746D63),
    ("lens_flare", TagGroup::LensFlare, 0x6C656E73),
    ("light", TagGroup::Light, 0x6C696768),
    ("light_volume", TagGroup::Lightning, 0x6D677332),
    ("lightning", TagGroup::LightVolume, 0x656C6563),
    ("material_effects", TagGroup::MaterialEffects, 0x666F6F74),
    ("meter", TagGroup::Meter, 0x6D657472),
    ("model", TagGroup::Model, 0x6D6F6465),
    ("model_animations", TagGroup::ModelAnimations, 0x616E7472),
    ("model_collision_geometry", TagGroup::ModelCollisionGeometry, 0x636F6C6C),
    ("multiplayer_scenario_description", TagGroup::MultiplayerScenarioDescription, 0x6D706C79),
    ("object", TagGroup::Object, 0x6F626A65),
    ("particle", TagGroup::Particle, 0x70617274),
    ("particle_system", TagGroup::ParticleSystem, 0x7063746C),
    ("physics", TagGroup::Physics, 0x70687973),
    ("placeholder", TagGroup::Placeholder, 0x706C6163),
    ("point_physics", TagGroup::PointPhysics, 0x70706879),
    ("preferences_network_game", TagGroup::PreferencesNetworkGame, 0x6E677072),
    ("projectile", TagGroup::Projectile, 0x70726F6A),
    ("scenario", TagGroup::Scenario, 0x73636E72),
    ("scenario_structure_bsp", TagGroup::ScenarioStructureBSP, 0x73627370),
    ("scenery", TagGroup::Scenery, 0x7363656E),
    ("shader", TagGroup::Shader, 0x73686472),
    ("shader_environment", TagGroup::ShaderEnvironment, 0x73656E76),
    ("shader_model", TagGroup::ShaderModel, 0x736F736F),
    ("shader_transparent_chicago", TagGroup::ShaderTransparentChicago, 0x73636869),
    ("shader_transparent_chicago_extended", TagGroup::ShaderTransparentChicagoExtended, 0x73636578),
    ("shader_transparent_generic", TagGroup::ShaderTransparentGeneric, 0x736F7472),
    ("shader_transparent_glass", TagGroup::ShaderTransparentGlass, 0x73676C61),
    ("shader_transparent_meter", TagGroup::ShaderTransparentMeter, 0x736D6574),
    ("shader_transparent_plasma", TagGroup::ShaderTransparentPlasma, 0x73706C61),
    ("shader_transparent_water", TagGroup::ShaderTransparentWater, 0x73776174),
    ("sky", TagGroup::Sky, 0x736B7920),
    ("sound", TagGroup::Sound, 0x736E6421),
    ("sound_environment", TagGroup::SoundEnvironment, 0x736E6465),
    ("sound_looping", TagGroup::SoundLooping, 0x6C736E64),
    ("sound_scenery", TagGroup::SoundScenery, 0x73736365),
    ("spheroid", TagGroup::Spheroid, 0x626F6F6D),
    ("string_list", TagGroup::StringList, 0x73747223),
    ("tag_collection", TagGroup::TagCollection, 0x74616763),
    ("ui_widget_collection", TagGroup::UIWidgetCollection, 0x536F756C),
    ("ui_widget_definition", TagGroup::UIWidgetDefinition, 0x44654C61),
    ("unicode_string_list", TagGroup::UnicodeStringList, 0x75737472),
    ("unit", TagGroup::Unit, 0x756E6974),
    ("unit_hud_interface", TagGroup::UnitHUDInterface, 0x756E6869),
    ("vehicle", TagGroup::Vehicle, 0x76656869),
    ("virtual_keyboard", TagGroup::VirtualKeyboard, 0x76636B79),
    ("weapon", TagGroup::Weapon, 0x77656170),
    ("weapon_hud_interface", TagGroup::WeaponHUDInterface, 0x77706869),
    ("weather_particle_system", TagGroup::WeatherParticleSystem, 0x7261696E),
    ("wind", TagGroup::Wind, 0x77696E64),
    ("zz_<none>", TagGroup::_None, 0xFFFFFFFF),
];

impl TagGroupFn for TagGroup {
    fn as_str(&self) -> &'static str {
        ALL_GROUPS[*self as usize].0
    }

    fn from_str(str: &str) -> Option<TagGroup> {
        match ALL_GROUPS.binary_search_by(|probe| probe.0.cmp(str)) {
            Ok(n) => Some(ALL_GROUPS[n].1),
            Err(_) => None
        }
    }

    fn as_fourcc(&self) -> FourCC {
        ALL_GROUPS[*self as usize].2
    }

    fn from_fourcc(fourcc: FourCC) -> Option<TagGroup> {
        for i in ALL_GROUPS {
            if i.2 == fourcc {
                return Some(i.1)
            }
        }
        None
    }

    fn none() -> TagGroup {
        TagGroup::_None
    }
}

impl Default for TagGroup {
    fn default() -> Self {
        TagGroup::_None
    }
}

#[cfg(test)]
mod tests {
    use super::{ALL_GROUPS, TagGroup, TagGroupFn};

    // Check if tag groups are sorted to ensure binary searching works.
    #[test]
    fn test_tag_groups_are_sorted() {
        for i in 0..ALL_GROUPS.len()-1 {
            let this = ALL_GROUPS[i];
            let next = ALL_GROUPS[i + 1];
            assert!(this.0 < next.0, "{} is not < {} (extension)", this.0, next.0);
            assert!(this.1 < next.1, "{} is not < {} (enum value)", this.0, next.0);
        }
        assert_eq!(TagGroup::_None as usize + 1, ALL_GROUPS.len(), "One or more groups are not in ALL_GROUPS!");

        // Test actually getting the values
        for i in ALL_GROUPS {
            assert_eq!(i.0, i.1.as_str());

            assert_eq!(i.1, TagGroup::from_str(i.0).unwrap());
            assert_eq!(i.1, TagGroup::from_fourcc(i.2).unwrap());

            assert_eq!(i.2, i.1.as_fourcc());
        }
    }
}
