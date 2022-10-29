use std::process::ExitCode;
use std::path::Path;
use ringhopper::engines::h1::definitions::{TagCollection, TagCollectionTag, UIWidgetCollection};
use ringhopper::engines::h1::*;
use crate::cmd::*;
use ringhopper::types::tag::TagGroupFn;
use macros::terminal::*;
use crate::file::*;
use ringhopper::error::{ErrorMessage, ErrorMessageResult};
use strings::get_compiled_string;

macro_rules! make_collection_tag {
    ($parser:ty, $input:expr) => {{
        let input = $input;

        // Put it together
        let mut collection = <$parser>::default();
        for l in input.lines() {
            if l.is_empty() { continue }
            collection.tags.blocks.push(TagCollectionTag {
                reference: TagReference::from_full_path(l)?
            });
        }
        Ok(collection)
    }}
}

pub fn collection_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args, &[], &[get_compiled_string!("arguments.specifier.tag_without_group")], executable, verb.get_description(), ArgumentConstraints::new().needs_data().needs_tags())?;

    let tags = Path::new(&parsed_args.named["tags"][0]);
    let data = Path::new(&parsed_args.named["data"][0]);
    let internal_path = &parsed_args.extra[0];
    let internal_path_path = Path::new(&internal_path).to_owned();
    let data_path = data.join(internal_path_path.clone()).with_extension("txt");
    let file_data = read_file(&data_path)?;

    // Parse as UTF-8
    let input = match std::str::from_utf8(&file_data) {
        Ok(n) => n.to_owned(),
        Err(error) => {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_parsing_file"), error=error, file=data_path.display())));
        }
    };

    // Make the tag collection tag
    let (output_extension, output_tag) = match *verb {
        Verb::TagCollection => (TagGroup::TagCollection.as_str(), make_collection_tag!(TagCollection, &input)?.into_tag_file()?),
        Verb::UICollection => (TagGroup::UIWidgetCollection.as_str(), make_collection_tag!(UIWidgetCollection, &input)?.into_tag_file()?),
        _ => unreachable!()
    };

    // Done
    let tag_path = tags.join(internal_path_path.clone()).with_extension(output_extension);
    make_parent_directories(&tag_path)?;
    write_file(&tag_path, &output_tag)?;

    println_success!(get_compiled_string!("engine.h1.verbs.unicode-strings.saved_file"), file=tag_path.display());
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use ringhopper::engines::h1::definitions::*;
    use ringhopper::engines::h1::*;
    use ringhopper::error::ErrorMessageResult;

    #[test]
    fn test_make_tag_collection_tag() {
        // Input should handle forward slashes, backslashes, mixed slashes, and empty lines.
        let input = "
weapons/pistol/pistol.weapon
levels\\test\\bloodgulch\\bloodgulch.scenario

weapons/assault rifle\\assault rifle.weapon";

        let collection = (|| -> ErrorMessageResult<TagCollection> {
            make_collection_tag!(TagCollection, input)
        })().unwrap();

        assert_eq!(3, collection.tags.blocks.len());
        assert_eq!("weapons\\pistol\\pistol.weapon", collection.tags[0].reference.get_path_with_extension());
        assert_eq!("levels\\test\\bloodgulch\\bloodgulch.scenario", collection.tags[1].reference.get_path_with_extension());
        assert_eq!("weapons\\assault rifle\\assault rifle.weapon", collection.tags[2].reference.get_path_with_extension());

        let collection_error = (|| -> ErrorMessageResult<TagCollection> {
            make_collection_tag!(TagCollection, "this input is invalid")
        })();

        assert!(collection_error.is_err());
    }
}
