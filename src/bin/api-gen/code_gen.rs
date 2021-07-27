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
use std::io::prelude::*;

use crate::alias::VppJsApiAlias;
use crate::basetypes::{maxSizeUnion, sizeof_alias, sizeof_struct};
use crate::enums::VppJsApiEnum;
use crate::file_schema::VppJsApiFile;
use crate::message::VppJsApiMessage;
use crate::parser_helper::{camelize_ident, get_ident, get_type};
use crate::types::VppJsApiType;
use crate::types::{VppJsApiFieldSize, VppJsApiMessageFieldDef};


pub fn gen_code(code: &VppJsApiFile, name: &str, api_definition: &mut Vec<(String, String)>) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/[a-z_0-9]*.api.json").unwrap();
    }
    
    // Do imports
    
    println!("{}", name);
    let fileName = RE
        .find(&name)
        .unwrap()
        .as_str()
        .trim_end_matches(".api.json");
    println!("{}", fileName);
    let mut file = File::create(format!("src/{}.rs", fileName)).unwrap();
    file.write_all(code.generate_code(name, api_definition).as_bytes()).unwrap();

    println!("Generated code for {}", fileName);
}
