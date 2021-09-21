#![allow(
    dead_code,
    unused_mut,
    unused_variables,
    unused_must_use,
    non_camel_case_types,
    unused_imports
)]
use env_logger::filter;
use lazy_static::lazy_static;
use linked_hash_map::LinkedHashMap;
use regex::Regex;
use std::fmt::format;
use std::fs::File;
use std::fs;
use std::io::prelude::*;

use crate::alias::VppJsApiAlias;
use crate::basetypes::{maxSizeUnion, sizeof_alias, sizeof_struct};
use crate::enums::VppJsApiEnum;
use crate::file_schema::VppJsApiFile;
use crate::message::VppJsApiMessage;
use crate::parser_helper::{camelize_ident, get_ident, get_type};
use crate::types::VppJsApiType;
use crate::types::{VppJsApiFieldSize, VppJsApiMessageFieldDef};

pub fn gen_code_file(code: &VppJsApiFile, name: &str, api_definition: &mut Vec<(String, String)>){
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[a-z_0-9]*.api.json").unwrap();
    }
    let fileName = RE
        .find(&name)
        .unwrap()
        .as_str()
        .trim_end_matches(".api.json");
        let mut file = File::create(format!(".././{}.rs",fileName)).unwrap();
    file.write_all(code.generate_code(name, api_definition).as_bytes())
        .unwrap();
    println!("Generated {}.rs", fileName.trim_start_matches("/"));
}
pub fn gen_code(
    code: &VppJsApiFile,
    name: &str,
    api_definition: &mut Vec<(String, String)>,
    packageName: &str,
) {
    // Using Regex to extract output filename
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/[a-z_0-9]*.api.json").unwrap();
    }
    let fileName = RE
        .find(&name)
        .unwrap()
        .as_str()
        .trim_end_matches(".api.json");
    let mut file = File::create(format!(".././{}/src/{}.rs", packageName, fileName)).unwrap();
    file.write_all(code.generate_code(name, api_definition).as_bytes())
        .unwrap();

    println!("Generated {}.rs", fileName.trim_start_matches("/"));
}

pub fn create_cargo_toml(packageName: &str) {
    println!("Generating Cargo file");
    let mut code = String::new();
    code.push_str("[package] \n");
    code.push_str(&format!("name = \"{}\" \n", packageName));
    code.push_str("version = \"0.1.0\" \n");
    code.push_str("authors = [\"Andrew Yourtchenko <ayourtch@gmail.com>\"] \n");
    code.push_str("edition = \"2018\" \n\n");

    code.push_str("[dev-dependencies] \n");
    code.push_str("trybuild = {version = \"1.0\", features = [\"diff\"]} \n\n");

    code.push_str("[dependencies] \n");
    code.push_str("serde = { version = \"1.0\", features = [\"derive\"] } \n");
    code.push_str("serde_json = \"1.0\" \n");
    code.push_str("clap = \"3.0.0-beta.2\" \n");
    code.push_str("strum = \"*\" \n");
    code.push_str("strum_macros = \"*\" \n");
    code.push_str("log = \"*\" \n");
    code.push_str("env_logger = \"*\" \n");
    code.push_str("linked-hash-map = { version = \"*\", features = [\"serde_impl\"] } \n");
    code.push_str("convert_case = \"*\" \n");
    code.push_str("serde_repr = \"0.1\" \n");
    code.push_str("typenum = \"*\" \n");
    code.push_str("bincode = \"1.2.1\" \n");
    code.push_str("serde_yaml = \"0.8\" \n");
    code.push_str("vpp-api-encoding = {git=\"https://github.com/ayourtch/vpp-api-encoding\", branch=\"main\" } \n");
    code.push_str("vpp-api-transport = { git=\"https://github.com/ayourtch/vpp-api-transport/\", branch=\"main\" } \n");
    code.push_str("lazy_static = \"1.4.0\" \n");
    code.push_str("regex = \"1\" \n");
    code.push_str("syn ={ version= \"1.0\", features=[\"extra-traits\",\"full\"]} \n");
    code.push_str("quote = \"1.0\" \n");
    code.push_str("proc-macro2 = \"1.0.26\" \n");
    code.push_str(
        "vpp-api-macros = {git=\"https://github.com/ayourtch/vpp-api-macros\", branch=\"main\"} \n",
    );

    let mut file = File::create(format!(".././{}/Cargo.toml", packageName)).unwrap();
    file.write_all(code.as_bytes()).unwrap();
}

pub fn generate_lib_file(api_files: &LinkedHashMap<String, VppJsApiFile>, packageName: &str) {
    let mut code = String::new();
    code.push_str("pub mod reqrecv; \n");
    for (name, f) in api_files.clone() {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"/[a-z_0-9]*.api.json").unwrap();
        }
        let fileName = RE
            .find(&name)
            .unwrap()
            .as_str()
            .trim_end_matches(".api.json");
        code.push_str(&format!("pub mod {}; \n", fileName.trim_start_matches("/")));
    }
    let mut file = File::create(format!(".././{}/src/lib.rs", packageName)).unwrap();
    file.write_all(code.as_bytes()).unwrap();
    // println!("{}", code);
}
pub fn copy_file_with_fixup(example_file: &str, packageName: &str, target_name: &str){
    let data = fs::read_to_string(example_file).unwrap();
    let updated_test = data.replace("vpp_api_gen", packageName); 
    let mut file = File::create(format!(".././{}/{}", packageName, target_name)).unwrap();
    file.write_all(updated_test.as_bytes()).unwrap();
}
