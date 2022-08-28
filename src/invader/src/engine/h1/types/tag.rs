use crate::types::{String32};
use crate::{ErrorMessageResult, ErrorMessage};
use super::TagGroup;

use strings::get_compiled_string;

/// FourCC "blam" used for tag file headers.
pub const BLAM_FOURCC: u32 = 0x626C616D;

/// Header used for tag files
pub struct TagFileHeader {
    /// Old tag ID. Unread.
    pub old_tag_id: u32,

    /// Old tag name. Unread.
    pub old_tag_name: String32,

    /// Tag group. Must be verified.
    pub tag_group: TagGroup,

    /// CRC32 checksum of the data after the header. Used for building cache files and possibly other things.
    pub crc32: u32,

    /// Equals 0x40, the size of the header. Probably unread.
    pub header_length: u32,

    /// Version of the tag group. Must be verified. See [TagFileHeader::version_for_group].
    pub tag_group_version: u16,

    /// Equals 255. Probably unread.
    pub something_255: u16,

    /// Equals [BLAM_FOURCC]. Must be verified.
    pub blam_fourcc: u32
}

impl TagFileHeader {
    /// Validate the header, returning an error with an explanation if it is invalid.
    pub fn validate(&self) -> ErrorMessageResult<()> {
        match self.validate_encapsulate() {
            Ok(n) => Ok(n),
            Err(n) => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.tag.header.error"), reason=n.to_string())))
        }
    }

    fn validate_encapsulate(&self) -> ErrorMessageResult<()> {
        if self.blam_fourcc != BLAM_FOURCC {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.tag.header.error_reason_blam_invalid"), blam=BLAM_FOURCC, other=self.blam_fourcc)))
        }

        let expected_version = TagFileHeader::version_for_group(self.tag_group);
        if self.tag_group_version != expected_version {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.tag.header.error_reason_version_unsupported"), group=self.tag_group, version=expected_version, other=self.tag_group_version)))
        }

        Ok(())
    }

    /// Get the version supported for a tag group.
    pub fn version_for_group(group: TagGroup) -> u16 {
        match group {
            TagGroup::Actor => 2,
            TagGroup::ModelAnimations => 4,
            TagGroup::Biped => 3,
            TagGroup::Bitmap => 7,
            TagGroup::Contrail => 3,
            TagGroup::Effect => 4,
            TagGroup::Equipment => 2,
            TagGroup::Item => 2,
            TagGroup::ItemCollection => 0,
            TagGroup::DamageEffect => 6,
            TagGroup::LensFlare => 2,
            TagGroup::Light => 3,
            TagGroup::SoundLooping => 3,
            TagGroup::GBXModel => 5,
            TagGroup::Globals => 3,
            TagGroup::Model => 4,
            TagGroup::ModelCollisionGeometry => 1,
            TagGroup::Particle => 2,
            TagGroup::ParticleSystem => 4,
            TagGroup::Physics => 4,
            TagGroup::Placeholder => 2,
            TagGroup::PreferencesNetworkGame => 2,
            TagGroup::Projectile => 5,
            TagGroup::ScenarioStructureBsp => 5,
            TagGroup::Scenario => 2,
            TagGroup::ShaderEnvironment => 2,
            TagGroup::Sound => 4,
            TagGroup::ShaderModel => 2,
            TagGroup::ShaderTransparentWater => 2,
            TagGroup::CameraTrack => 2,
            TagGroup::Unit => 2,
            TagGroup::VirtualKeyboard => 2,
            TagGroup::Weapon => 2,
            TagGroup::WeaponHUDInterface => 2,
            _ => 1
        }
    }
}
