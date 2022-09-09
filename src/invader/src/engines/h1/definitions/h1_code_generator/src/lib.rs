extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;
use syn::{parse_macro_input, Expr, Lit};

use std::fs::File;
use std::path::Path;
use std::io::Read;

extern crate serde_json;
use serde_json::Value;

/// Load the definitions json file.
#[proc_macro]
pub fn load_json_def(input: TokenStream) -> TokenStream {
    let literal = match parse_macro_input!(input as Expr) {
        Expr::Lit(n) => n,
        _ => panic!("expected a string literal here")
    };

    let path = match literal.lit {
        Lit::Str(n) => Path::new(env!("CARGO_MANIFEST_DIR")).join("json").join(Path::new(&n.value())),
        _ => panic!("expected a string literal here")
    };

    let mut file = File::open(Path::new(&path)).unwrap();
    let mut data = Vec::<u8>::new();
    file.read_to_end(&mut data).unwrap();
    std::mem::drop(file);

    let data_string = std::str::from_utf8(&data).unwrap();

    let array = match serde_json::from_str::<Value>(data_string) {
        Ok(json) => match json {
            Value::Array(array) => array,
            _ => panic!("strings.json is the wrong JSON type")
        },
        Err(e) => panic!("Can't parse strings.json! {}", e)
    };

    let mut stream = TokenStream::new();

    // Go through each element in the JSON
    for i in array {
        let object = i.as_object().unwrap();
        let object_name = object.get("name").unwrap().as_str().unwrap();
        let object_type = object.get("type").unwrap().as_str().unwrap();

        if object_type == "enum" {
            stream.extend(format!("pub type {object_name} = u16;").parse::<TokenStream>().unwrap());
        }
        else if object_type == "bitfield" {
            stream.extend(format!("pub type {object_name} = u{width};", width = object.get("width").unwrap().as_u64().unwrap()).parse::<TokenStream>().unwrap());
        }
        else if object_type == "struct" {
            // Generate Rust code to define our struct and implementation
            let tag_size = object.get("size").unwrap().as_u64().unwrap();
            let mut all_fields_defined = String::new();

            // If we inherit anything, handle that too
            let mut from_tag_code;
            let mut into_tag_code;
            match object.get("inherits") {
                Some(n) => {
                    let inherited_object = n.as_str().unwrap();
                    all_fields_defined += &format!("pub base_struct: {inherited_object},");
                    from_tag_code = format!("new_object.base_struct = {inherited_object}::from_tag(data, at, struct_end, cursor)?; let mut local_cursor = {inherited_object}::tag_size();");
                    into_tag_code = format!("self.base_struct.into_tag(data, at, struct_end)?; let mut local_cursor = {inherited_object}::tag_size();");
                },
                None => {
                    from_tag_code = format!("let mut local_cursor = 0usize;");
                    into_tag_code = format!("let mut local_cursor = 0usize;");
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
                let field_name_written = field_name.replace(" ", "_").replace("'", "").replace(":", "").replace("(", "").replace(")", "");
                let field_name_written = match field_name_written.as_str() {
                    "type" | "loop" => format!("_{field_name_written}"),
                    _ => field_name_written
                };

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

                // Put it in the struct
                all_fields_defined += &format!("pub {field_name_written}: {field_type_struct},");

                // Is this swapped?
                let swapped = match f.get("little_endian").unwrap_or(&Value::Bool(false)).as_bool().unwrap() {
                    true => "_swapped",
                    false => ""
                };

                // Is this cache only?
                let cache_only = f.get("cache_only").unwrap_or(&Value::Bool(false)).as_bool().unwrap();

                let mut write_serialization_code = |type_suffix: &str| {
                    //from_tag_code += &format!("println!(\"...reading {field_name_written} - {field_type_struct}, AT: 0x{{at:08X}} + 0x{{local_cursor:08X}} / SE: 0x{{struct_end:08X}} / SZ: 0x{{size:08X}}\", at=at, local_cursor=local_cursor, struct_end=struct_end, size=data.len());");

                    // If it is cache only, we read it from the tag but don't keep it
                    if cache_only {
                        from_tag_code += &format!("let _ = {field_type_written_expression}::from_tag{swapped}(data, at + local_cursor, struct_end, cursor)?;");
                    }

                    // Otherwise we serialize it normally
                    else {
                        into_tag_code += &format!("self.{field_name_written}{type_suffix}.into_tag{swapped}(data, at + local_cursor, struct_end)?;");
                        from_tag_code += &format!("new_object.{field_name_written}{type_suffix} = {field_type_written_expression}::from_tag{swapped}(data, at + local_cursor, struct_end, cursor)?;");
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
            }

            // Define the struct
            stream.extend(format!("#[derive(Default, Clone, PartialEq)] pub struct {object_name} {{ {all_fields_defined} }}").parse::<TokenStream>().unwrap());

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
                    debug_assert_eq!({tag_size}, local_cursor);
                    Ok(())
                }}
                fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<{object_name}> {{
                    let mut new_object = {object_name}::default();
                    {from_tag_code}
                    debug_assert_eq!({tag_size}, local_cursor);
                    Ok(new_object)
                }}
            }}");
            stream.extend(parsing_code.parse::<TokenStream>().unwrap());
        }
    }

    stream
}
