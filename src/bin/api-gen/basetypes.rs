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
use crate::enums::VppJsApiEnum;
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
pub fn maxSizeUnion(unions: &VppJsApiType, file: &VppJsApiFile) -> u8 {
    unions.fields.iter()
    .map(|x| field_size(&x,&file))
    .fold(0,|mut max,x|{
        if x>max{
            max=x
        }
        max
    })
}
pub fn field_size(fields: &VppJsApiMessageFieldDef, file: &VppJsApiFile) -> u8{
    if fields.ctype.starts_with("vl_api_"){
        find_type(&file, &fields.ctype)      
    }
    else {
        let typ = basetypes::ctoSizeR(&fields.ctype);
        typ.basetypeSizes()
    }
}
pub fn find_type(file: &VppJsApiFile, name: &str) -> u8{
    let mut totalsize: u8 = 0; 
    let mut count = 0; 
    while count == 0 {
        for x in 0..file.types.len(){
            if name.trim_start_matches("vl_api_").trim_end_matches("_t") == file.types[x].type_name {
                totalsize = sizeof_struct(&file, &file.types[x]);
                count = count+1; 
                break;
            }
            
        }
        for x in 0..file.enums.len(){
            if name.trim_start_matches("vl_api_").trim_end_matches("_t") == file.enums[x].name {
                totalsize = sizeof_enum(&file.enums[x]);
                count = count+1; 
                break;
            }           
        }
        for x in file.aliases.keys() {
            if name.trim_start_matches("vl_api_").trim_end_matches("_t") == x {
                totalsize = sizeof_alias(&file.aliases[x], &file);
                count = count + 1; 
                break;
            }
        }
        for x in 0..file.unions.len(){
            if name.trim_start_matches("vl_api_").trim_end_matches("_t") == file.unions[x].type_name {
                totalsize = maxSizeUnion(&file.unions[x], &file);
                count = count+1; 
                break;
            }
            
        }
    }
    if count == 0 {
        println!("Could not find type");
    }
    totalsize
}
pub fn sizeof_enum(enums: &VppJsApiEnum) -> u8{
    match &enums.info.enumtype {
        Some(len) => {
            let typ = basetypes::ctoSizeR(len);
            typ.basetypeSizes()
        },
        _ => 32,
    }
}
pub fn sizeof_struct(file: &VppJsApiFile, structs: &VppJsApiType) -> u8{
    let mut totalsize:u8 = 0; 
    for x in &structs.fields{
        totalsize = totalsize + field_size(&x, &file);
    }
    totalsize
}

pub fn sizeof_alias(alias: &VppJsApiAlias, file: &VppJsApiFile) -> u8{
    if alias.ctype.starts_with("vl_api_") {
        find_type(&file, &alias.ctype)
    } else {
        match alias.length {
            Some(len) => {
                let typ = basetypes::ctoSizeR(&alias.ctype);
                let totalsize = typ.basetypeSizes() * len as u8;
                totalsize
            }
            _ => {
                let typ = basetypes::ctoSizeR(&alias.ctype);
                typ.basetypeSizes()
            }
        }
    }
}
