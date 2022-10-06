extern crate proc_macro;
use proc_macro::TokenStream;

use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::path::Path;
use std::io::Read;

extern crate serde_json;
use serde_json::{Value, Map};

/// Load the definitions json files.
#[proc_macro]
pub fn load_json_def(_: TokenStream) -> TokenStream {
    let mut stream = TokenStream::new();

    // Parse each json one by one
    let mut jsons = HashMap::<String, Map<String, Value>>::new();
    for path in read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("json")).unwrap() {
        let path = path.as_ref().unwrap().path();
        let mut file = File::open(&path).unwrap();
        let mut data = Vec::<u8>::new();
        file.read_to_end(&mut data).unwrap();
        std::mem::drop(file);

        let data_string = std::str::from_utf8(&data).unwrap();
        let object = match serde_json::from_str::<Value>(data_string) {
            Ok(json) => match json {
                Value::Object(array) => array,
                _ => panic!("{:?} is the wrong JSON type", path)
            },
            Err(e) => panic!("Can't parse {:?}! {}", path, e)
        };
        jsons.insert(path.file_name().unwrap().to_str().unwrap().to_owned(), object);
    }

    let mut engine_array = String::new();

    for filename in jsons.keys() {
        fn get_value(key: &str, base_filename: &str, jsons: &HashMap::<String, Map<String, Value>>) -> Option<Value> {
            let json = jsons.get(base_filename).unwrap();
            let derives = json.get("derives");

            // Get this value and the derived value.
            let this_value = match json.get(key) {
                Some(Value::Null) => None,
                Some(n) => Some(n),
                None => None
            };

            let derived_value = if let Some(v) = derives {
                get_value(key, &v.as_str().unwrap(), jsons)
            }
            else {
                None
            };

            // If we don't have a value here, return the derived value and call it a day.
            if this_value.is_none() {
                // Set fallback to false
                if key == "fallback" {
                    return Some(Value::Bool(false));
                }

                // Don't pass through shorthands
                if key == "shorthand" {
                    return None;
                }

                return derived_value;
            }

            // If we aren't looking for required tags or we have no derived value, return this value and call it a day.
            if key != "required_tags" {
                return this_value.map(|f| f.to_owned());
            }

            // Merge required tags
            let mut this_value = this_value.unwrap().to_owned().as_object().unwrap().to_owned();

            macro_rules! insert_if_not_inserted {
                ($key:expr) => {{
                    if !this_value.contains_key($key) {
                        this_value.insert($key.to_owned(), Value::Array(Vec::new()));
                    }
                }}
            }

            insert_if_not_inserted!("all");
            insert_if_not_inserted!("singleplayer");
            insert_if_not_inserted!("multiplayer");
            insert_if_not_inserted!("user_interface");
            insert_if_not_inserted!("singleplayer_demo");
            insert_if_not_inserted!("multiplayer_demo");
            insert_if_not_inserted!("user_interface_demo");

            // If we have nothing derived, we're done!
            if derived_value.is_none() {
                return Some(Value::Object(this_value));
            }

            // Otherwise, merge them.
            let mut derived_value = derived_value.unwrap().as_object().unwrap().to_owned();

            macro_rules! merge_it_all {
                ($key:expr) => {{
                    derived_value.get_mut($key).unwrap().as_array_mut().unwrap().append(this_value.get_mut("all").unwrap().as_array_mut().unwrap());
                }}
            }

            merge_it_all!("all");
            merge_it_all!("singleplayer");
            merge_it_all!("multiplayer");
            merge_it_all!("user_interface");
            merge_it_all!("singleplayer_demo");
            merge_it_all!("multiplayer_demo");
            merge_it_all!("user_interface_demo");

            Some(Value::Object(derived_value))
        }

        fn parse_int_value(value: Value) -> u64 {
            match value {
                Value::String(string) => {
                    if string.starts_with("0x") {
                        u64::from_str_radix(&string[2..], 16).unwrap()
                    }
                    else {
                        panic!()
                    }
                },
                Value::Number(number) => number.as_u64().unwrap(),
                _ => panic!("expected unsigned int or hex")
            }
        }

        fn parse_optional_string(value: Option<Value>) -> Option<String> {
            value.map(|v| v.as_str().unwrap().to_owned())
        }

        fn encapsulate_optional_string(string: Option<String>) -> String {
            match string {
                Some(n) => format!("Some({})", encapsulate_string(n)),
                None => "None".to_owned()
            }
        }

        fn encapsulate_string(string: String) -> String {
            format!("\"{string}\"")
        }

        let name = encapsulate_string(get_value("name", filename, &jsons).unwrap().as_str().unwrap().to_owned());

        let version = encapsulate_optional_string(parse_optional_string(get_value("version", filename, &jsons)));
        let shorthand = encapsulate_optional_string(parse_optional_string(get_value("shorthand", filename, &jsons)));
        let build = encapsulate_optional_string(parse_optional_string(get_value("build", filename, &jsons)));
        let script_compile_target = match get_value("script_compile_target", filename, &jsons).unwrap().as_str().unwrap() {
            "xbox" => "CompileTarget::HaloCEXboxNTSC",
            "mcc-cea" => "CompileTarget::HaloCEA",
            "gbx-custom" => "CompileTarget::HaloCustomEdition",
            "gbx-retail" => "CompileTarget::HaloCEGBX",
            "gbx-demo" => "CompileTarget::HaloCEGBXDemo",
            n => panic!("Unknown script compile target {}", n)
        };

        let fallback = get_value("fallback", filename, &jsons).unwrap().as_bool().unwrap();
        let required_tags = get_value("required_tags", filename, &jsons).unwrap().as_object().unwrap().to_owned();
        let max_script_nodes = parse_int_value(get_value("max_script_nodes", &filename, &jsons).unwrap());
        let cache_file_version = parse_int_value(get_value("cache_file_version", &filename, &jsons).unwrap());
        let bsps_occupy_tag_space = get_value("bsps_occupy_tag_space", filename, &jsons).unwrap_or(Value::Bool(false)).as_bool().unwrap();
        let max_tag_space = parse_int_value(get_value("max_tag_space", &filename, &jsons).unwrap());

        let max_cache_file_size = get_value("max_cache_file_size", filename, &jsons).unwrap().as_object().unwrap().to_owned();
        let max_cache_file_size_user_interface = parse_int_value(max_cache_file_size.get("user_interface").unwrap().to_owned());
        let max_cache_file_size_singleplayer = parse_int_value(max_cache_file_size.get("singleplayer").unwrap().to_owned());
        let max_cache_file_size_multiplayer = parse_int_value(max_cache_file_size.get("multiplayer").unwrap().to_owned());

        let base_memory_address = get_value("base_memory_address", filename, &jsons).unwrap().as_object().unwrap().to_owned();
        let base_memory_address_value = parse_int_value(base_memory_address.get("value").unwrap().to_owned());
        let base_memory_address_type = base_memory_address.get("type").unwrap().as_str().unwrap().to_owned();

        let base_memory_address_tokens = match base_memory_address_type.as_str() {
            "fixed" => format!("BaseMemoryAddressType::Fixed({base_memory_address_value})"),
            "inferred" => format!("BaseMemoryAddressType::Inferred({base_memory_address_value})"),
            n => panic!("unknown base memory address type {}", n)
        };

        let mut required_tags_tokens = String::new();
        for (target, tags) in required_tags {
            let mut new_tags = String::new();
            for i in tags.as_array().unwrap() {
                new_tags += &format!("\"{}\",", i.as_str().unwrap().replace("\\", "\\\\"));
            }
            required_tags_tokens += &format!("{target}: &[{new_tags}],");
        }
        let required_tags_tokens = format!("RequiredTags {{ {required_tags_tokens} }}");

        let tokens = format!("
            EngineTarget {{
                name: {name},
                version: {version},
                shorthand: {shorthand},
                build: {build},
                fallback: {fallback},
                max_script_nodes: {max_script_nodes},
                cache_file_version: {cache_file_version},
                bsps_occupy_tag_space: {bsps_occupy_tag_space},
                max_tag_space: {max_tag_space},
                max_cache_file_size_user_interface: {max_cache_file_size_user_interface},
                max_cache_file_size_singleplayer: {max_cache_file_size_singleplayer},
                max_cache_file_size_multiplayer: {max_cache_file_size_multiplayer},
                base_memory_address: {base_memory_address_tokens},
                required_tags: {required_tags_tokens},
                script_compile_target: {script_compile_target}
            }},
        ");

        engine_array += &tokens;
    }

    stream.extend(format!("pub const ALL_TARGETS: &'static [EngineTarget] = &[{engine_array}];").parse::<TokenStream>());

    stream
}
