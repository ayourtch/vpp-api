#![allow(
    dead_code,
    unused_mut,
    unused_variables,
    unused_must_use,
    non_camel_case_types,
    unused_imports,
    non_snake_case
)]

use crate::alias::VppJsApiAlias;
use crate::file_schema::VppJsApiFile;
use crate::types::VppJsApiMessageFieldDef;
use crate::types::VppJsApiType;
pub enum basetypes {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F64,
    BOOL,
    STRING,
}

impl basetypes {
    fn basetypeSizes(&self) -> u8 {
        match self {
            basetypes::U8 => 1,
            basetypes::I8 => 1,
            basetypes::U16 => 2,
            basetypes::I16 => 2,
            basetypes::U32 => 4,
            basetypes::I32 => 4,
            basetypes::U64 => 8,
            basetypes::I64 => 8,
            basetypes::F64 => 8,
            basetypes::BOOL => 1,
            basetypes::STRING => 1,
        }
    }
    fn rustTypes(&self) -> &str {
        match self {
            basetypes::U8 => "u8",
            basetypes::I8 => "i8",
            basetypes::U16 => "u16",
            basetypes::I16 => "i16",
            basetypes::U32 => "u32",
            basetypes::I32 => "i32",
            basetypes::U64 => "u64",
            basetypes::I64 => "i64",
            basetypes::F64 => "f64",
            basetypes::BOOL => "bool",
            basetypes::STRING => "string",
        }
    }
    fn ctoSize(size: &str) -> basetypes {
        match size {
            "uint8" => basetypes::U8,
            "int8" => basetypes::I8,
            "uint16" => basetypes::U16,
            "int16" => basetypes::I16,
            "uint32" => basetypes::U32,
            "int32" => basetypes::I32,
            "uint64" => basetypes::U64,
            "int64" => basetypes::I64,
            "float64" => basetypes::F64,
            "bool" => basetypes::BOOL,
            "string" => basetypes::STRING,
            _ => basetypes::U8,
        }
    }
    fn ctoSizeR(size: &str) -> basetypes {
        match size {
            "u8" => basetypes::U8,
            "i8" => basetypes::I8,
            "u16" => basetypes::U16,
            "i16" => basetypes::I16,
            "u32" => basetypes::U32,
            "i32" => basetypes::I32,
            "u64" => basetypes::U64,
            "i64" => basetypes::I64,
            "f64" => basetypes::F64,
            "bool" => basetypes::BOOL,
            "string" => basetypes::STRING,
            _ => basetypes::U8,
        }
    }
}
pub fn maxSizeUnion(unions: &VppJsApiType) {
    // dbg!(&unions);
    for x in &unions.fields {
        dbg!(&x.ctype);
    }
}
// Size of enum
// Size of struct
// Size of Aliases
pub fn sizeof_alias(alias: &VppJsApiAlias) {
    // dbg!(&alias);
    if alias.ctype.starts_with("vl_api_") {
        // println!("Need to calculate struct here")
        // Search for this struct and then find the size
    } else {
        match alias.length {
            Some(len) => {
                let typ = basetypes::ctoSizeR(&alias.ctype);
                let totalsize = typ.basetypeSizes() * len as u8;
                // println!("Size is {}",totalsize);
            }
            _ => {
                let typ = basetypes::ctoSizeR(&alias.ctype);
                // println!("Size is {}", typ as u8);
            }
        }
    }
}
pub fn sizeof_struct(structs: &VppJsApiType, apifile: &VppJsApiFile) {
    dbg!(&structs);
    let mut totalsize: u8 = 0;
    for x in 0..structs.fields.len() {
        totalsize = totalsize + sizeof_field(&structs.fields[x], &apifile);
    }
    println!("Size is {}", totalsize);
}
pub fn sizeof_field(fields: &VppJsApiMessageFieldDef, apifile: &VppJsApiFile) -> u8 {
    // dbg!(&fields);
    if fields.ctype.starts_with("vl_api_") {
        find_struct(&fields.ctype, apifile);
        0 // Incomplete
    } else {
        let typ = basetypes::ctoSizeR(&fields.ctype);
        // println!("Size is {}", typ.basetypeSizes());
        typ.basetypeSizes()
    }
}
// Find the struct
pub fn find_struct(name: &str, apifile: &VppJsApiFile) {
    for x in 0..apifile.types.len() {
        if apifile.types[x].type_name == name {
            println!("Found the struct");
            sizeof_struct(&apifile.types[x], &apifile);
            break;
        }
    }
}
// Size of enum
