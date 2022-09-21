use engines::ExitCode;
use std::path::Path;
use crate::engines::h1::definitions::{TagCollection, TagCollectionTag};
use crate::engines::h1::types::TagGroup;
use crate::engines::h1::{TagFileSerializeFn, TagReference};
use crate::cmd::*;
use crate::types::tag::TagGroupFn;
use invader_macros::terminal::*;
use crate::file::*;
use crate::error::{ErrorMessage, ErrorMessageResult};
use strings::get_compiled_string;

fn make_collection_tag(file_data: &[u8], data_path: &Path) -> ErrorMessageResult<TagCollection> {
    // Parse as UTF-8
    let string = match std::str::from_utf8(&file_data) {
        Ok(n) => n.to_owned(),
        Err(error) => {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_parsing_file"), error=error, file=data_path.display())));
        }
    };

    // Put it together
    let mut collection = TagCollection::default();
    for l in string.lines() {
        if l.is_empty() { continue }
        collection.tags.blocks.push(TagCollectionTag {
            reference: TagReference::from_path_with_extension(l)?
        });
    }

    Ok(collection)
}

pub fn collection_verb(verb: &Verb, args: &[&str], executable: &str) -> ErrorMessageResult<ExitCode> {
    let parsed_args = ParsedArguments::parse_arguments(args, &[], &[get_compiled_string!("arguments.specifier.tag_without_group")], executable, verb.get_description(), ArgumentConstraints::new().needs_data().needs_tags())?;

    let tags = Path::new(&parsed_args.named.get("tags").unwrap()[0]);
    let data = Path::new(&parsed_args.named.get("data").unwrap()[0]);
    let internal_path = &parsed_args.extra[0];
    let internal_path_path = Path::new(&internal_path).to_owned();
    let data_path = data.join(internal_path_path.clone()).with_extension("txt");
    let file_data = read_file(&data_path)?;

    // Make the tag collection tag
    let collection = make_collection_tag(&file_data, &data_path)?;

    // Done
    let output_tag = collection.into_tag_file()?;
    let tag_path = tags.join(internal_path_path.clone()).with_extension(TagGroup::TagCollection.as_str());
    write_file(&tag_path, &output_tag)?;

    println_success!(get_compiled_string!("engine.h1.verbs.unicode-strings.saved_file"), file=tag_path.display());
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_make_tag_collection_tag() {
        // Input should handle forward slashes, backslashes, mixed slashes, and empty lines.
        let input = b"
weapons/pistol/pistol.weapon
levels\\test\\bloodgulch\\bloodgulch.scenario

weapons/assault rifle\\assault rifle.weapon";
        let collection = super::make_collection_tag(input, std::path::Path::new("some_input.txt")).unwrap();
        assert_eq!(3, collection.tags.blocks.len());
        assert_eq!("weapons\\pistol\\pistol.weapon", collection.tags[0].reference.get_path_with_extension());
        assert_eq!("levels\\test\\bloodgulch\\bloodgulch.scenario", collection.tags[1].reference.get_path_with_extension());
        assert_eq!("weapons\\assault rifle\\assault rifle.weapon", collection.tags[2].reference.get_path_with_extension());

        assert!(super::make_collection_tag(b"this input is invalid", std::path::Path::new("bad_input.txt")).is_err());
    }
}
