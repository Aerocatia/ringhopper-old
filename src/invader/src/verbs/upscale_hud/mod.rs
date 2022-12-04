use ringhopper::engines::h1::{TagGroup, TagFileSerializeFn};
use ringhopper::engines::h1::definitions::*;
use ringhopper_proc::*;
use std::num::NonZeroUsize;
use std::process::ExitCode;
use crate::cmd::*;
use ringhopper::error::{ErrorMessageResult, ErrorMessage};
use ringhopper::file::*;
use ringhopper::types::*;
use crate::file::*;
use macros::terminal::*;

pub fn upscale_hud_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(
        args,
        &[],
        &[get_compiled_string!("arguments.specifier.tag_batch_with_group")],
        executable,
        verb.get_description(),
        ArgumentConstraints::new().needs_tags().multiple_tags_directories().uses_threads()
    )?;

    Ok(super::do_with_batching_threaded(upscale_hud_tag, &parsed_args.extra[0], None, &str_slice_to_path_vec(&parsed_args.named["tags"]), parsed_args.threads, TagFile::uses_batching(&parsed_args.extra[0]))?.exit_code())
}

fn upscale_hud_tag(tag: &TagFile, log_mutex: super::LogMutex, _available_threads: NonZeroUsize, batched: &bool) -> ErrorMessageResult<bool> {
    let group = tag.tag_path.get_group();

    let upscale_fn: fn(&TagFile) -> ErrorMessageResult<Vec<u8>> = match tag.tag_path.get_group() {
        TagGroup::GrenadeHUDInterface => upscale_grenade_hud_interface,
        TagGroup::HUDGlobals => upscale_hud_globals,
        TagGroup::WeaponHUDInterface => upscale_weapon_hud_interface,
        TagGroup::UnitHUDInterface => upscale_unit_hud_interface,
        _ => {
            if !batched {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.upscale-hud.error_unable_to_upscale_tag"), tag=tag.tag_path, group=group)))
            }
            return Ok(false)
        },
    };

    write_file(&tag.file_path, &upscale_fn(tag)?)?;

    let l = log_mutex.lock();
    println_success!(get_compiled_string!("engine.h1.verbs.upscale-hud.upscaled_tag"), tag=tag.tag_path);
    drop(l);

    Ok(true)
}

fn modify_scaling(width_scale: &mut f32, height_scale: &mut f32, offset: &mut Point2DInt, flags: &mut HUDInterfaceScalingFlags) {
    if flags.use_high_res_scale {
        flags.use_high_res_scale = false;
    }
    else {
        if *width_scale <= 0.0 {
            *width_scale = 2.0;
        }
        else {
            *width_scale *= 2.0;
        }

        if *height_scale <= 0.0 {
            *height_scale = 2.0;
        }
        else {
            *height_scale *= 2.0;
        }
    }

    safe_double(&mut offset.x);
    safe_double(&mut offset.y);
}

fn upscale_weapon_hud_interface(path: &TagFile) -> ErrorMessageResult<Vec<u8>> {
    let mut tag = *WeaponHUDInterface::from_tag_file(&read_file(&path.file_path)?)?.data;
    for e in &mut tag.static_elements {
        modify_scaling(&mut e.width_scale, &mut e.height_scale, &mut e.anchor_offset, &mut e.scaling_flags);
    }
    for e in &mut tag.meter_elements {
        modify_scaling(&mut e.width_scale, &mut e.height_scale, &mut e.anchor_offset, &mut e.scaling_flags);
    }
    for e in &mut tag.number_elements {
        modify_scaling(&mut e.width_scale, &mut e.height_scale, &mut e.anchor_offset, &mut e.scaling_flags);
    }
    for e in &mut tag.crosshairs {
        for o in &mut e.crosshair_overlays {
            o.scaling_flags.use_high_res_scale = false; // this flag is always unused, so honoring this flag may break some HUDs
            modify_scaling(&mut o.width_scale, &mut o.height_scale, &mut o.anchor_offset, &mut o.scaling_flags);
        }
    }
    for e in &mut tag.overlay_elements {
        for o in &mut e.overlays {
            modify_scaling(&mut o.width_scale, &mut o.height_scale, &mut o.anchor_offset, &mut o.scaling_flags);
        }
    }
    tag.into_tag_file()
}

fn upscale_unit_hud_interface(path: &TagFile) -> ErrorMessageResult<Vec<u8>> {
    let mut tag = *UnitHUDInterface::from_tag_file(&read_file(&path.file_path)?)?.data;
    modify_scaling(&mut tag.hud_background_width_scale, &mut tag.hud_background_height_scale, &mut tag.hud_background_anchor_offset, &mut tag.hud_background_scaling_flags);

    modify_scaling(&mut tag.health_panel_background_width_scale, &mut tag.health_panel_background_height_scale, &mut tag.health_panel_background_anchor_offset, &mut tag.health_panel_background_scaling_flags);
    modify_scaling(&mut tag.health_panel_meter_width_scale, &mut tag.health_panel_meter_height_scale, &mut tag.health_panel_meter_anchor_offset, &mut tag.health_panel_meter_scaling_flags);

    modify_scaling(&mut tag.shield_panel_background_width_scale, &mut tag.shield_panel_background_height_scale, &mut tag.shield_panel_background_anchor_offset, &mut tag.shield_panel_background_scaling_flags);
    modify_scaling(&mut tag.shield_panel_meter_width_scale, &mut tag.shield_panel_meter_height_scale, &mut tag.shield_panel_meter_anchor_offset, &mut tag.shield_panel_meter_scaling_flags);

    modify_scaling(&mut tag.motion_sensor_background_width_scale, &mut tag.motion_sensor_background_height_scale, &mut tag.motion_sensor_background_anchor_offset, &mut tag.motion_sensor_background_scaling_flags);
    modify_scaling(&mut tag.motion_sensor_foreground_width_scale, &mut tag.motion_sensor_foreground_height_scale, &mut tag.motion_sensor_foreground_anchor_offset, &mut tag.motion_sensor_foreground_scaling_flags);
    modify_scaling(&mut tag.motion_sensor_center_width_scale, &mut tag.motion_sensor_center_height_scale, &mut tag.motion_sensor_center_anchor_offset, &mut tag.motion_sensor_center_scaling_flags);

    for e in &mut tag.overlays {
        modify_scaling(&mut e.width_scale, &mut e.height_scale, &mut e.anchor_offset, &mut e.scaling_flags);
    }

    for e in &mut tag.meters {
        modify_scaling(&mut e.background_width_scale, &mut e.background_height_scale, &mut e.background_anchor_offset, &mut e.background_scaling_flags);
        modify_scaling(&mut e.meter_width_scale, &mut e.meter_height_scale, &mut e.meter_anchor_offset, &mut e.meter_scaling_flags);
    }

    tag.into_tag_file()
}

fn upscale_grenade_hud_interface(path: &TagFile) -> ErrorMessageResult<Vec<u8>> {
    let mut tag = *GrenadeHUDInterface::from_tag_file(&read_file(&path.file_path)?)?.data;

    for e in &mut tag.total_grenades_overlays {
        modify_scaling(&mut e.width_scale, &mut e.height_scale, &mut e.anchor_offset, &mut e.scaling_flags);
    }

    modify_scaling(&mut tag.background_width_scale, &mut tag.background_height_scale, &mut tag.background_anchor_offset, &mut tag.background_scaling_flags);
    modify_scaling(&mut tag.total_grenades_background_width_scale, &mut tag.total_grenades_background_height_scale, &mut tag.total_grenades_background_anchor_offset, &mut tag.total_grenades_background_scaling_flags);
    modify_scaling(&mut tag.total_grenades_numbers_width_scale, &mut tag.total_grenades_numbers_height_scale, &mut tag.total_grenades_numbers_anchor_offset, &mut tag.total_grenades_numbers_scaling_flags);

    safe_double(&mut tag.messaging_information_width_offset);
    safe_double(&mut tag.messaging_information_offset_from_reference_corner.x);
    safe_double(&mut tag.messaging_information_offset_from_reference_corner.y);

    tag.into_tag_file()
}

fn upscale_hud_globals(path: &TagFile) -> ErrorMessageResult<Vec<u8>> {
    let mut tag = *HUDGlobals::from_tag_file(&read_file(&path.file_path)?)?.data;

    modify_scaling(&mut tag.width_scale, &mut tag.height_scale, &mut tag.anchor_offset, &mut tag.scaling_flags);

    safe_double(&mut tag.hud_damage_top_offset);
    safe_double(&mut tag.hud_damage_left_offset);
    safe_double(&mut tag.hud_damage_right_offset);
    safe_double(&mut tag.hud_damage_bottom_offset);

    tag.top_offset *= 2.0;
    tag.left_offset *= 2.0;
    tag.right_offset *= 2.0;
    tag.bottom_offset *= 2.0;

    tag.into_tag_file()
}

fn safe_double(value: &mut i16) {
    *value = value.saturating_mul(2);
}
