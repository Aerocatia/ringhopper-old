use ringhopper_proc::*;
use std::num::NonZeroUsize;
use std::process::ExitCode;
use crate::cmd::*;
use ringhopper::engines::h1::TagGroup;
use ringhopper::engines::h1::definitions::*;
use ringhopper::error::*;
use ringhopper::file::*;
use crate::file::*;
use macros::terminal::*;
use ringhopper::engines::h1::*;
use ringhopper::types::tag::TagGroupFn;
use std::convert::{From, TryFrom};

type ConversionFunctionTuple = (TagGroup, fn (&[u8]) -> ErrorMessageResult<Vec<u8>>);

fn convert_object_to_object<T: ObjectSuperFn + TagFileSerializeFn, U: ObjectSuperFn + TagFileSerializeFn + Default>(data: &[u8]) -> ErrorMessageResult<Vec<u8>> {
    let mut to_object = U::default();
    *to_object.get_base_object_mut() = T::from_tag_file(data)?.data.get_base_object().to_owned();
    to_object.into_tag_file()
}

fn convert_unit_to_unit<T: UnitSuperFn + TagFileSerializeFn, U: UnitSuperFn + TagFileSerializeFn + Default>(data: &[u8]) -> ErrorMessageResult<Vec<u8>> {
    let mut to_object = U::default();
    *to_object.get_base_unit_mut() = T::from_tag_file(data)?.data.get_base_unit().to_owned();
    to_object.into_tag_file()
}

fn convert_item_to_item<T: ItemSuperFn + TagFileSerializeFn, U: ItemSuperFn + TagFileSerializeFn + Default>(data: &[u8]) -> ErrorMessageResult<Vec<u8>> {
    let mut to_object = U::default();
    *to_object.get_base_item_mut() = T::from_tag_file(data)?.data.get_base_item().to_owned();
    to_object.into_tag_file()
}

fn convert_device_to_device<T: DeviceSuperFn + TagFileSerializeFn, U: DeviceSuperFn + TagFileSerializeFn + Default>(data: &[u8]) -> ErrorMessageResult<Vec<u8>> {
    let mut to_object = U::default();
    *to_object.get_base_device_mut() = T::from_tag_file(data)?.data.get_base_device().to_owned();
    to_object.into_tag_file()
}

macro_rules! add_object_conversion_functions {
    ($from:tt, $unit_fn:tt, $item_fn:tt, $device_fn:tt) => {
        (
            TagGroup::$from,
            &[
                (TagGroup::Biped, $unit_fn::<$from, Biped>),
                (TagGroup::Vehicle, $unit_fn::<$from, Vehicle>),
                (TagGroup::Weapon, $item_fn::<$from, Weapon>),
                (TagGroup::Equipment, $item_fn::<$from, Equipment>),
                (TagGroup::Garbage, $item_fn::<$from, Garbage>),
                (TagGroup::Projectile, convert_object_to_object::<$from, Projectile>),
                (TagGroup::Scenery, convert_object_to_object::<$from, Scenery>),
                (TagGroup::DeviceMachine, $device_fn::<$from, DeviceMachine>),
                (TagGroup::DeviceControl, $device_fn::<$from, DeviceControl>),
                (TagGroup::DeviceLightFixture, $device_fn::<$from, DeviceLightFixture>),
                (TagGroup::Placeholder, convert_object_to_object::<$from, Placeholder>),
                (TagGroup::SoundScenery, convert_object_to_object::<$from, SoundScenery>),
            ]
        )
    };
}

const CONVERSION_FUNCTION_GROUPS: &'static [(TagGroup, &'static [ConversionFunctionTuple])] = &[
    add_object_conversion_functions!(Biped, convert_unit_to_unit, convert_object_to_object, convert_object_to_object),
    add_object_conversion_functions!(Vehicle, convert_unit_to_unit, convert_object_to_object, convert_object_to_object),
    add_object_conversion_functions!(Weapon, convert_object_to_object, convert_item_to_item, convert_object_to_object),
    add_object_conversion_functions!(Equipment, convert_object_to_object, convert_item_to_item, convert_object_to_object),
    add_object_conversion_functions!(Garbage, convert_object_to_object, convert_item_to_item, convert_object_to_object),
    add_object_conversion_functions!(Projectile, convert_object_to_object, convert_object_to_object, convert_object_to_object),
    add_object_conversion_functions!(Scenery, convert_object_to_object, convert_object_to_object, convert_object_to_object),
    add_object_conversion_functions!(DeviceMachine, convert_object_to_object, convert_object_to_object, convert_device_to_device),
    add_object_conversion_functions!(DeviceControl, convert_object_to_object, convert_object_to_object, convert_device_to_device),
    add_object_conversion_functions!(DeviceLightFixture, convert_object_to_object, convert_object_to_object, convert_device_to_device),
    add_object_conversion_functions!(Placeholder, convert_object_to_object, convert_object_to_object, convert_object_to_object),
    add_object_conversion_functions!(SoundScenery, convert_object_to_object, convert_object_to_object, convert_object_to_object),

    (TagGroup::Model, &[(TagGroup::GBXModel, |data| {
        let model = *Model::from_tag_file(data)?.data;
        model.check_for_extraction_bugs()?;
        GBXModel::from(model).into_tag_file()
    })]),
    (TagGroup::GBXModel, &[(TagGroup::Model, |data| {
        let model = *GBXModel::from_tag_file(data)?.data;
        model.check_for_extraction_bugs()?;
        Model::try_from(model)?.into_tag_file()
    })]),

    (TagGroup::ShaderTransparentChicagoExtended, &[(TagGroup::ShaderTransparentChicago, |data| { ShaderTransparentChicago::from(*ShaderTransparentChicagoExtended::from_tag_file(data)?.data).into_tag_file() })]),
    (TagGroup::ShaderTransparentChicago, &[(TagGroup::ShaderTransparentChicagoExtended, |data| { ShaderTransparentChicagoExtended::from(*ShaderTransparentChicago::from_tag_file(data)?.data).into_tag_file() })])
];

#[derive(Copy, Clone)]
struct ConvertOptions {
    output_group: TagGroup,
    overwrite: bool,
    batching: bool
}

fn convert_tag(tag: &TagFile, log_mutex: super::LogMutex, _available_threads: NonZeroUsize, options: &ConvertOptions) -> ErrorMessageResult<bool> {
    let mut output_path = tag.file_path.to_owned();
    output_path.set_extension(options.output_group.as_str());

    let input_group = tag.tag_path.get_group();

    // We cannot convert one group to the same group.
    if input_group == options.output_group {
        return Ok(false)
    }

    for fg in CONVERSION_FUNCTION_GROUPS {
        if fg.0 == input_group {
            for f in fg.1 {
                if f.0 == options.output_group {
                    // Does our output already exist?
                    if output_path.exists() && !options.overwrite {
                        let l = log_mutex.lock();
                        println!(get_compiled_string!("engine.h1.verbs.convert.skipped_tag"), tag=tag.tag_path);
                        drop(l);
                        return Ok(false)
                    }

                    let file_data = read_file(&tag.file_path)?;
                    let result = f.1(&file_data)?;
                    write_file(&output_path, &result)?;
                    let l = log_mutex.lock();
                    println_success!(get_compiled_string!("engine.h1.verbs.convert.converted_tag"), tag=tag.tag_path);
                    drop(l);
                    return Ok(true);
                }
            }
            break
        }
    }

    if !options.batching {
        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.convert.unable_to_convert_tag"), tag=tag.tag_path, output_group=options.output_group)))
    }

    Ok(false)
}

pub fn convert_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args, &[], &[get_compiled_string!("arguments.specifier.tag_batch_with_group"), "new-group"], executable, verb.get_description(), ArgumentConstraints::new().needs_tags().multiple_tags_directories().can_overwrite())?;
    let overwrite = parsed_args.named.get("overwrite").is_some();
    let output_group = match TagGroup::from_str(&parsed_args.extra[1]) {
        Some(n) => n,
        None => {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.types.error_path_group_invalid"), potential_group=parsed_args.extra[1])));
        }
    };
    let tag_path = &parsed_args.extra[0];
    let batching = TagFile::uses_batching(tag_path);
    let options = ConvertOptions { overwrite, output_group, batching };

    Ok(super::do_with_batching_threaded(convert_tag, &tag_path, None, &str_slice_to_path_vec(&parsed_args.named["tags"]), parsed_args.threads, options)?.exit_code())
}
