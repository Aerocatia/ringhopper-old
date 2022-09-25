/// Crate for getting strings used in Ringhopper's output.
///
/// These strings are compile-time and can be used in `format!`.

extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;
use syn::{parse_macro_input, Expr, Lit};

extern crate serde_json;
use serde_json::Value;

const STRINGS_JSON_DATA: &'static str = include_str!("strings.json");

fn get_string(name: String) -> String {
    let map = match serde_json::from_str::<Value>(STRINGS_JSON_DATA) {
        Ok(json) => match json {
            Value::Object(map) => map,
            _ => panic!("strings.json is the wrong JSON type")
        },
        Err(e) => panic!("Can't parse strings.json! {}", e)
    };

    match map.get(&name) {
        Some(n) => match n {
            Value::String(s) => s.to_string(),
            _ => panic!("\"{}\" does not correspond to a string in strings.json!", name)
        },
        None => panic!("\"{}\" does not correspond to anything in strings.json", name)
    }
}

/// Get the string compiled into strings.json.
#[proc_macro]
pub fn get_compiled_string(input: TokenStream) -> TokenStream {
    let literal = match parse_macro_input!(input as Expr) {
        Expr::Lit(n) => n,
        _ => panic!("expected a string literal here")
    };

    let string = match literal.lit {
        Lit::Str(n) => n,
        _ => panic!("expected a string literal here")
    }.value();

    format!("\"{}\"", get_string(string).replace("\\", "\\\\").replace("\"", "\\\"")).parse().unwrap()
}
