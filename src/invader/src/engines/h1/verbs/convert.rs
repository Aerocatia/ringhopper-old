use strings::*;
use crate::engines::ExitCode;
use crate::cmd::*;
use crate::engines::h1::TagGroup;
use crate::engines::h1::definitions::*;
use crate::error::ErrorMessage;
use crate::file::*;
use crate::terminal::*;
use crate::error::ErrorMessageResult;
use crate::engines::h1::{TagFileSerializeFn, ObjectSuperFn};
use crate::types::tag::TagGroupFn;
use std::convert::{From, TryFrom};

type ConversionFunctionTuple = (TagGroup, TagGroup, fn (&[u8]) -> ErrorMessageResult<Vec<u8>>);

fn convert_object_to_scenery<T: ObjectSuperFn + TagFileSerializeFn>(data: &[u8]) -> ErrorMessageResult<Vec<u8>> {
    Scenery {
        base_struct: BasicObject {
            base_struct: T::from_tag_file(data)?.data.get_base_object().to_owned(),
            more_flags: BaseObjectFlags::default()
        }
    }.into_tag_file()
}

const CONVERSION_FUNCTIONS: &'static [ConversionFunctionTuple] = &[
    (TagGroup::Biped, TagGroup::Scenery, convert_object_to_scenery::<Biped>),
    (TagGroup::Vehicle, TagGroup::Scenery, convert_object_to_scenery::<Vehicle>),
    (TagGroup::Weapon, TagGroup::Scenery, convert_object_to_scenery::<Weapon>),
    (TagGroup::Equipment, TagGroup::Scenery, convert_object_to_scenery::<Equipment>),
    (TagGroup::Garbage, TagGroup::Scenery, convert_object_to_scenery::<Garbage>),
    (TagGroup::Projectile, TagGroup::Scenery, convert_object_to_scenery::<Projectile>),
    (TagGroup::DeviceMachine, TagGroup::Scenery, convert_object_to_scenery::<DeviceMachine>),
    (TagGroup::DeviceControl, TagGroup::Scenery, convert_object_to_scenery::<DeviceControl>),
    (TagGroup::DeviceLightFixture, TagGroup::Scenery, convert_object_to_scenery::<DeviceLightFixture>),
    (TagGroup::Placeholder, TagGroup::Scenery, convert_object_to_scenery::<Placeholder>),
    (TagGroup::SoundScenery, TagGroup::Scenery, convert_object_to_scenery::<SoundScenery>),

    (TagGroup::Model, TagGroup::GBXModel, |data| { GBXModel::from(*Model::from_tag_file(data)?.data).into_tag_file() }),
    (TagGroup::GBXModel, TagGroup::Model, |data| { Model::try_from(*GBXModel::from_tag_file(data)?.data)?.into_tag_file() }),

    (TagGroup::ShaderTransparentChicagoExtended, TagGroup::ShaderTransparentChicago, |data| { ShaderTransparentChicago::from(*ShaderTransparentChicagoExtended::from_tag_file(data)?.data).into_tag_file() })
];

fn convert_tag(path: &std::path::Path, output_group: TagGroup) -> ErrorMessageResult<bool> {
    let mut output_path = path.to_owned();
    output_path.set_extension(output_group.as_str());

    // Does our output already exist?
    if output_path.exists() {
        return Ok(false)
    }

    let file_data = read_file(path)?;
    let input_group = TagGroup::from_str(path.extension().unwrap().to_str().unwrap()).unwrap();

    let result = (|| {
        for f in CONVERSION_FUNCTIONS {
            if f.0 == input_group && f.1 == output_group {
                return f.2(&file_data);
            }
        }
        Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.convert.unable_to_convert_tag"), input_group=input_group, output_group=output_group)))
    })()?;

    write_file(&output_path, &result)?;
    Ok(true)
}

pub fn convert_verb(verb: &Verb, args: &[&str], executable: &str) -> ExitCode {
    let parsed_args = try_parse_arguments!(args, &[], &[get_compiled_string!("arguments.specifier.tag_batch_with_group"), "new-group"], executable, verb.get_description(), ArgumentConstraints::new().needs_tags().multiple_tags_directories());

    let tags = match TagFile::from_tag_path_batched(&str_slice_to_path_vec(&parsed_args.named["tags"]), &parsed_args.extra[0], None) {
        Ok(n) => n,
        Err(e) => panic!("{}", e)
    };

    let group = match TagGroup::from_str(&parsed_args.extra[1]) {
        Some(n) => n,
        None => {
            eprintln_error!(get_compiled_string!("engine.types.error_path_group_invalid"), potential_group=parsed_args.extra[1]);
            return ExitCode::FAILURE;
        }
    };

    let mut count = 0usize;
    let total = tags.len();
    for i in tags {
        match convert_tag(&i.file_path, group) {
            Ok(written) => {
                if written {
                    println_success!(get_compiled_string!("engine.h1.verbs.convert.converted_tag"), tag=i.tag_path);
                }
                else {
                    println!(get_compiled_string!("engine.h1.verbs.convert.skipped_tag"), tag=i.tag_path);
                }
                count += 1;
            },
            Err(e) => eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.convert.error_could_not_convert_tag"), tag=i.tag_path, error=e)
        }
    }

    if total == 0 {
        eprintln_error_pre!(get_compiled_string!("file.error_no_tags_found"));
    }
    else if count == 0 {
        eprintln_error_pre!(get_compiled_string!("engine.h1.verbs.convert.error_no_tags_converted"), total=total);
    }
    else if count != total {
        println_warn!(get_compiled_string!("engine.h1.verbs.convert.converted_some_tags"), count=count, total=total);
    }
    else {
        println_success!(get_compiled_string!("engine.h1.verbs.convert.converted_all_tags"), count=count);
    }

    if count > 0 {
        ExitCode::SUCCESS
    }
    else {
        ExitCode::FAILURE
    }
}
