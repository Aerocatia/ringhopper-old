extern crate proc_macro;
use proc_macro::TokenStream;

use std::fs::{read_dir, File};
use std::path::Path;
use std::io::Read;

extern crate serde_json;
use serde_json::{Value, Map};

// Convert a tag group extension (e.g. "unit_hud_interface" -> "UnitHUDInterface")
fn tag_group_extension_to_struct(group: &str) -> String {
    let mut new_group = group.to_owned().replace("gbxmodel", "GBXModel")
                                        .replace("bsp", "BSP")
                                        .replace("ui_", "UI_")
                                        .replace("hud_", "HUD_");
    while let Some(s) = new_group.find("_") {
        new_group.replace_range(s..s+2, &new_group[s+1..s+2].to_uppercase());
    }
    new_group.replace_range(0..1, &new_group[0..1].to_uppercase());
    new_group
}

// Format the string into being 'friendly' for use as an identifier in Rust code.
fn format_tag_field_name(string: &str) -> String {
    let field_name_written = string.replace(" ", "_").replace("-", "_").replace("'", "").replace(":", "").replace("(", "").replace(")", "");

    // Keyword
    let field_name_written = match field_name_written.as_str() {
        "type" | "loop" => format!("_{field_name_written}"),
        _ => field_name_written
    };

    // Number-only bitfields
    if field_name_written.chars().next().unwrap().is_numeric() {
        format!("_{field_name_written}")
    }
    else {
        field_name_written
    }
}

/// Load the definitions json files.
#[proc_macro]
pub fn load_json_def(_: TokenStream) -> TokenStream {
    let mut stream = TokenStream::new();
    let mut group_to_struct = Vec::<String>::new();

    for path in read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("json")).unwrap() {
        let mut file = File::open(&path.as_ref().unwrap().path()).unwrap();
        let mut data = Vec::<u8>::new();
        file.read_to_end(&mut data).unwrap();
        std::mem::drop(file);

        let data_string = std::str::from_utf8(&data).unwrap();

        let array = match serde_json::from_str::<Value>(data_string) {
            Ok(json) => match json {
                Value::Array(array) => array,
                _ => panic!("{:?} is the wrong JSON type", path)
            },
            Err(e) => panic!("Can't parse {:?}! {}", path, e)
        };

        // Go through each element in the JSON
        for i in array {
            let object = i.as_object().unwrap();
            let object_name = object.get("name").unwrap().as_str().unwrap();
            let object_type = object.get("type").unwrap().as_str().unwrap();

            if object_type == "enum" {
                stream.extend(format!("pub type {object_name} = u16;").parse::<TokenStream>().unwrap());
            }
            else if object_type == "bitfield" {
                let width = object.get("width").unwrap().as_u64().unwrap();
                let width_bytes = width / 8;
                let mut fields = String::new();
                let mut into_uint_code = String::new();
                let mut from_uint_code = String::new();

                let mut current_value = 1u32;
                let mut tag_mask = 0xFFFFFFFFu32;
                let mut value_mask = 0u32;

                for f in object.get("fields").unwrap().as_array().unwrap() {
                    let o = match f.as_object() {
                        Some(n) => n.to_owned(),
                        None => {
                            let mut m = Map::<String, Value>::new();
                            m.insert("name".to_owned(), Value::String(f.as_str().unwrap().to_owned()));
                            m
                        }
                    };

                    let name = format_tag_field_name(o.get("name").unwrap().as_str().unwrap());
                    fields += &format!("pub {name}: bool,");

                    // Set these masks so we know what to read/write
                    if f.get("cache_only").unwrap_or(&Value::Bool(false)).as_bool().unwrap() {
                        tag_mask &= !current_value; // exclude cache only bits from being read/written from tag files
                    }
                    value_mask |= current_value;

                    into_uint_code += &format!("return_value |= {current_value}u{width} * (self.{name} as u{width});");
                    from_uint_code += &format!("{name}: (input_value & {current_value}u{width}) != 0,");

                    current_value <<= 1;
                }

                // Exclude all bits not actually used too.
                tag_mask &= value_mask;

                stream.extend(format!("
                #[derive(Default, Copy, Clone, PartialEq)]
                pub struct {object_name} {{
                    {fields}
                }}").parse::<TokenStream>().unwrap());

                stream.extend(format!("impl {object_name} {{
                    /// Get the numeric representation of the bitfield.
                    pub fn into_u{width}(&self) -> u{width} {{
                        let mut return_value = 0u{width};
                        {into_uint_code}
                        return_value
                    }}

                    /// Convert the number into a bitfield.
                    ///
                    /// Bits that do not exist on the bitfield are ignored.
                    pub fn from_u{width}(input_value: u{width}) -> {object_name} {{
                        {object_name} {{
                            {from_uint_code}
                        }}
                    }}
                }}").parse::<TokenStream>().unwrap());

                let parsing_code = format!("
                impl TagSerialize for {object_name} {{
                    fn tag_size() -> usize {{
                        {width_bytes}
                    }}
                    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {{
                        (self.into_u{width}() & {tag_mask}).into_tag(data, at, struct_end)
                    }}
                    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<{object_name}> {{
                        Ok({object_name}::from_u{width}(u{width}::from_tag(data, at, struct_end, cursor)? & {tag_mask}))
                    }}
                }}");
                stream.extend(parsing_code.parse::<TokenStream>().unwrap());
            }
            else if object_type == "struct" {
                // Check if we implement copy
                let mut implements_copy = true;

                // Generate Rust code to define our struct and implementation
                let tag_size = object.get("size").unwrap().as_u64().unwrap();
                let mut all_fields_defined = String::new();

                // If this is a group, note it here.
                match object.get("group") {
                    Some(n) => group_to_struct.push(tag_group_extension_to_struct(n.as_str().unwrap())),
                    None => ()
                }

                // If we inherit anything, handle that too
                let mut from_tag_code;
                let mut into_tag_code;
                match object.get("inherits") {
                    Some(n) => {
                        implements_copy = false; // can't determine this

                        let inherited_object = n.as_str().unwrap();
                        all_fields_defined += &format!("pub base_struct: {inherited_object},");
                        from_tag_code = format!("new_object.base_struct = {inherited_object}::from_tag(data, at, struct_end, cursor)?; let mut local_cursor = at + {inherited_object}::tag_size();");
                        into_tag_code = format!("self.base_struct.into_tag(data, at, struct_end)?; let mut local_cursor = at + {inherited_object}::tag_size();");
                    },
                    None => {
                        from_tag_code = format!("let mut local_cursor = at;");
                        into_tag_code = format!("let mut local_cursor = at;");
                    }
                }

                for f in object.get("fields").unwrap().as_array().unwrap() {
                    let field_type = f.get("type").unwrap().as_str().unwrap();

                    // If it is padding, we may have some special behavior here.
                    if field_type == "pad" {
                        let cursor_increment = format!("local_cursor += {};", f.get("size").unwrap().as_u64().unwrap());
                        from_tag_code += &cursor_increment;
                        into_tag_code += &cursor_increment;
                        continue
                    }

                    // Otherwise, let's do this
                    let field_name = f.get("name").unwrap().as_str().unwrap();
                    let field_name_written = format_tag_field_name(&field_name);

                    if field_type == "Reflexive" || field_type == "Data" || field_type == "TagReference" {
                        implements_copy = false;
                    }

                    let field_type_data_type = match field_type {
                        "Reflexive" => format!("Reflexive<{}>", f.get("struct").unwrap().as_str().unwrap()),
                        "int8" => "i8".to_owned(),
                        "int16" => "i16".to_owned(),
                        "int32" => "i32".to_owned(),
                        "uint8" => "u8".to_owned(),
                        "uint16" => "u16".to_owned(),
                        "uint32" => "u32".to_owned(),
                        "float" | "Angle" | "Fraction" => "f32".to_owned(),
                        f => f.to_owned()
                    };

                    // Here's the type
                    let field_type_data_type = if f.get("bounds").unwrap_or(&Value::Bool(false)).as_bool().unwrap() {
                        format!("Bounds<{field_type_data_type}>")
                    }
                    else {
                        field_type_data_type
                    };

                    // Next, do we need to make it an array?
                    let count = match f.get("count") {
                        Some(n) => n.as_u64().unwrap(),
                        None => 1
                    };

                    // To make sure Rust is happy, we put a '::' before any <'s
                    let field_type_written_expression = field_type_data_type.replace("<", "::<");

                    // Array?
                    let field_type_struct = match count {
                        1 => field_type_data_type,
                        _ => format!("[{field_type_data_type}; {count}]")
                    };

                    // Is this swapped?
                    let swapped = match f.get("little_endian").unwrap_or(&Value::Bool(false)).as_bool().unwrap() {
                        true => "_swapped",
                        false => ""
                    };

                    // Is this cache only?
                    let cache_only = f.get("cache_only").unwrap_or(&Value::Bool(false)).as_bool().unwrap();

                    let mut doc = f.get("comment").unwrap_or(&Value::String(String::new())).as_str().unwrap().to_owned();

                    // Write the serialization code
                    let mut write_serialization_code = |type_suffix: &str| {
                        //from_tag_code += &format!("println!(\"...reading {field_name_written} - {field_type_struct}, AT: 0x{{at:08X}} -> 0x{{local_cursor:08X}} / SE: 0x{{struct_end:08X}} / SZ: 0x{{size:08X}}\", at=at, local_cursor=local_cursor, struct_end=struct_end, size=data.len());");

                        // If it is cache only, if it is inconsequential (i.e. does not advance the data cursor), we ignore it. Otherwise, we read it from the tag but don't keep it.
                        // Either way, we do not generate any code to put it in a tag file.
                        if cache_only {
                            if field_type == "TagReference" || field_type == "Data" || field_type == "Reflexive" {
                                from_tag_code += &format!("{field_type_written_expression}::from_tag{swapped}(data, local_cursor, struct_end, cursor)?;");
                            }
                        }

                        // Otherwise we serialize it normally
                        else {
                            into_tag_code += &format!("self.{field_name_written}{type_suffix}.into_tag{swapped}(data, local_cursor, struct_end)?;");
                            from_tag_code += &format!("new_object.{field_name_written}{type_suffix} = {field_type_written_expression}::from_tag{swapped}(data, local_cursor, struct_end, cursor)?;");
                        }

                        let cursor_increment = format!("local_cursor += {field_type_written_expression}::tag_size();");
                        from_tag_code += &cursor_increment;
                        into_tag_code += &cursor_increment;
                    };

                    // One object, not an array
                    if count == 1 {
                        write_serialization_code("");
                    }

                    // Array
                    else {
                        for i in 0..count {
                            write_serialization_code(&format!("[{i}]"));
                        }
                    }

                    if field_type_struct == "TagReference" {
                        if !doc.is_empty() {
                            doc += "\n\n";
                        }

                        let groups_arr = f.get("groups").unwrap().as_array().unwrap();
                        let mut groups = Vec::<String>::new();

                        for g in groups_arr {
                            let appended_group: &'static [&'static str] = match g.as_str().unwrap() {
                                "unit"   => &["biped", "vehicle"],
                                "item"   => &["weapon", "equipment", "garbage"],
                                "device" => &["device_machine", "device_light_fixture", "device_control"],
                                "object" => &["biped", "vehicle",
                                              "weapon", "equipment", "garbage",
                                              "scenery",
                                              "device_machine", "device_light_fixture", "device_control",
                                              "placeholder",
                                              "sound_scenery"],

                                "model"  => &["gbxmodel", "model"],

                                "shader" => &[
                                    "shader_environment",
                                    "shader_model",
                                    "shader_transparent_chicago_extended",
                                    "shader_transparent_chicago",
                                    "shader_transparent_generic",
                                    "shader_transparent_glass",
                                    "shader_transparent_meter",
                                    "shader_transparent_plasma",
                                    "shader_transparent_water"
                                ],

                                n => {
                                    groups.push(n.to_owned());
                                    &[]
                                }
                            };

                            groups.reserve(appended_group.len());
                            for &i in appended_group {
                                groups.push(i.to_owned());
                            }
                        }

                        groups.dedup();
                        groups.sort();

                        if groups == ["*"] {
                            doc += &format!("Allowed groups: All\n");
                        }
                        else {
                            doc += &format!("Allowed groups: ");
                            let mut add_comma = false;

                            // Format groups into matching their equivalent group name
                            for g in &groups {
                                let new_group = tag_group_extension_to_struct(&g);

                                match add_comma {
                                    true => doc += ", ",
                                    false => add_comma = true
                                };

                                doc += &format!("[{new_group}](TagGroup::{new_group})");
                            }
                            doc += "\n";
                        }

                        // Allow default_group to be specified
                        let default_group = if groups == ["*"] {
                            "TagCollection".to_owned()
                        }
                        else if groups_arr[0].as_str().unwrap() == "model" {
                            "Model".to_owned()
                        }
                        else {
                            tag_group_extension_to_struct(&groups[0])
                        };

                        // Default the group
                        if !cache_only {
                            from_tag_code += &format!("if new_object.{field_name_written}.get_group() == TagGroup::_None {{ new_object.{field_name_written}.set_group(TagGroup::{default_group}); }}");
                        }
                    }

                    // Put the doc in the struct
                    if doc != "" {
                        doc = doc.replace("\"", "\\\"");
                        all_fields_defined += &format!("#[doc=\"{doc}\"] ")
                    }

                    // Put it in the struct
                    all_fields_defined += &format!("pub {field_name_written}: {field_type_struct},");
                }

                // Define the struct
                stream.extend(format!("#[derive(Default{}, Clone, PartialEq)] pub struct {object_name} {{ {all_fields_defined} }}", match implements_copy { true => ", Copy", false => "" } ).parse::<TokenStream>().unwrap());

                // Define parsing it too
                stream.extend(format!("
                impl TagBlockFn for {object_name} {{
                    fn field_count(&self) -> usize {{ todo!() }}
                    fn field_at_index(&self, _: usize) -> TagField {{ todo!() }}
                    fn field_at_index_mut(&mut self, _: usize) -> TagField{{ todo!() }}
                }}").parse::<TokenStream>().unwrap());

                // Next serializing code
                let parsing_code = format!("
                impl TagSerialize for {object_name} {{
                    fn tag_size() -> usize {{
                        {tag_size}
                    }}
                    fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {{
                        {into_tag_code}
                        debug_assert_eq!(at + {tag_size}, local_cursor, \"Size for {object_name} is wrong\");
                        Ok(())
                    }}
                    fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<{object_name}> {{
                        let mut new_object = {object_name}::default();
                        {from_tag_code}
                        debug_assert_eq!(at + {tag_size}, local_cursor, \"Size for {object_name} is wrong\");
                        Ok(new_object)
                    }}
                }}");
                stream.extend(parsing_code.parse::<TokenStream>().unwrap());
            }
        }
    }

    // Write functions for reading tags with TagFileSerializeFn
    let mut group_read_match_block = String::new();
    for group in group_to_struct {
        stream.extend(format!("impl TagFileSerializeFn for {group} {{
            fn from_tag_file(data: &[u8]) -> ErrorMessageResult<ParsedTagFile<Self>> {{
                let header = TagFileHeader::from_tag(data, 0, TAG_FILE_HEADER_LEN, &mut TAG_FILE_HEADER_LEN.clone())?;
                if header.tag_group == TagGroup::{group} {{
                    ParsedTagFile::from_tag(data)
                }}
                else {{
                    Err(ErrorMessage::AllocatedString(format!(get_compiled_string!(\"engine.h1.types.tag.header.error_reason_wrong_group\"), group_expected=\"{group}\", group_actual=header.tag_group.as_str())))
                }}
            }}
            fn into_tag_file(&self) -> ErrorMessageResult<Vec<u8>> {{
                ParsedTagFile::into_tag(self, TagGroup::{group})
            }}
        }}").parse::<TokenStream>());

        group_read_match_block += &format!("TagGroup::{group} => {{
            let tag_file = {group}::from_tag_file(data)?;
            Ok(ParsedTagFile {{
                header: tag_file.header,
                data: tag_file.data
            }})
        }},");
    }

    stream.extend(format!("
        /// Generic function for parsing a tag file for when knowing the tag group is not required.
        ///
        /// Returns an error if the tag could not be parsed.
        pub fn parse_tag_file(data: &[u8]) -> ErrorMessageResult<ParsedTagFile<dyn TagFileSerializeFn>> {{
        let header = TagFileHeader::from_tag(data, 0, TAG_FILE_HEADER_LEN, &mut TAG_FILE_HEADER_LEN.clone())?;
        match header.tag_group {{
            {group_read_match_block}
            n => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!(\"engine.h1.types.tag.header.error_reason_unparsable_group\"), group=n.as_str())))
        }}
    }}").parse::<TokenStream>());

    stream
}
