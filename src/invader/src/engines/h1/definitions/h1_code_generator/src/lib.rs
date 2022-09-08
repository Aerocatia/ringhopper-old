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
        Lit::Str(n) => Path::new(env!("CARGO_MANIFEST_DIR")).join("../json").join(Path::new(&n.value())),
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
            let mut from_tag_code = String::new();
            let mut into_tag_code = String::new();

            let mut field_count = 0usize;
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
                let field_name_written = field_name.replace(" ", "_");
                let field_name_written = if field_name_written == "type" {
                    "_type".to_owned()
                }
                else {
                    field_name_written
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
                let field_type_written = if f.get("bounds").unwrap_or(&Value::Bool(false)).as_bool().unwrap() {
                    format!("Bounds<{field_type_data_type}>")
                }
                else {
                    field_type_data_type
                };

                // Next, do we need to make it an array?
                let field_type_written = match f.get("count") {
                    Some(n) => format!("[{field_type_written}; {count}]", count=n.as_u64().unwrap()),
                    None => field_type_written
                };

                // To make sure Rust is happy, we put a '::' before any <'s
                let field_type_written_expression = field_type_written.replace("<", "::<");

                all_fields_defined += &format!("pub {field_name_written}: {field_type_written},");

                // Now let's serialize it
                into_tag_code += &format!("self.{field_name_written}.into_tag(data, at + local_cursor, struct_end)?;");
                from_tag_code += &format!("new_object.{field_name_written} = {field_type_written_expression}::from_tag(data, at + local_cursor, struct_end, cursor)?;");

                // Increment the cursor
                let cursor_increment = format!("local_cursor += {field_type_written_expression}::tag_size();");
                from_tag_code += &cursor_increment;
                into_tag_code += &cursor_increment;

                field_count += 1;
            }

            // Define the struct
            stream.extend(format!("#[derive(Default, Clone, PartialEq)] pub struct {object_name} {{ {all_fields_defined} }}").parse::<TokenStream>().unwrap());

            // Define parsing it too
            stream.extend(format!("
            impl TagBlockFn for {object_name} {{
                fn field_count(&self) -> usize {{ {field_count} }}
                fn field_at_index(&self, _: usize) -> TagField {{ todo!() }}
                fn field_at_index_mut(&mut self, _: usize) -> TagField{{ todo!() }}
            }}").parse::<TokenStream>().unwrap());

            // Next serializing code
            stream.extend(format!("
            impl TagSerialize for {object_name} {{
                fn tag_size() -> usize {{
                    {tag_size}
                }}
                fn into_tag(&self, data: &mut Vec<u8>, at: usize, struct_end: usize) -> ErrorMessageResult<()> {{
                    let mut local_cursor = 0usize;
                    {into_tag_code}
                    Ok(())
                }}
                fn from_tag(data: &[u8], at: usize, struct_end: usize, cursor: &mut usize) -> ErrorMessageResult<{object_name}> {{
                    let mut local_cursor = 0usize;
                    let mut new_object = {object_name}::default();
                    {from_tag_code}
                    Ok(new_object)
                }}
            }}").parse::<TokenStream>().unwrap());
        }
    }

    stream
}
