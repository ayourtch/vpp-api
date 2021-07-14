use std::fmt::format;
use std::fs::File;
use std::io::prelude::*;

use crate::alias::VppJsApiAlias;
use crate::enums::VppJsApiEnum;
use crate::file_schema::VppJsApiFile;
use crate::types::VppJsApiType;
use crate::message::VppJsApiMessage;
use crate::types::{VppJsApiMessageFieldDef, VppJsApiFieldSize};
use crate::parser_helper::{get_type, get_ident,camelize_ident};
use crate::basetypes::{maxSizeUnion, sizeof_alias, sizeof_struct};

pub fn gen_code(code: &VppJsApiFile){
    let mut preamble: String = format!("/* Autogenerated data. Do not edit */\n");
    preamble.push_str("#![allow(non_camel_case_types)]\n");
    preamble.push_str("use serde::{de::DeserializeOwned, Deserialize, Serialize};\n");
    preamble.push_str("use vpp_api_encoding::typ::*;\n");
    preamble.push_str("use vpp_api_transport::*;\n");
    preamble.push_str("use serde_repr::{Serialize_repr, Deserialize_repr};\n");
    for x in 0..code.types.len(){
        gen_structs(&code.types[x], &mut preamble, &code);
    }
    for x in 0..code.unions.len(){
        gen_union(&code.unions[x], &mut preamble);
    }
    for x in 0..code.enums.len(){
        gen_enum(&code.enums[x], &mut preamble);
    } 
    // gen_alias(&code.aliases[0]);
    
    for x in code.aliases.keys(){
        gen_alias(&code.aliases[x], x, &mut preamble);
    }
    for x in 0..code.messages.len(){
        gen_messages(&code.messages[x], &mut preamble);
    }
    // println!("{}",preamble);
    // dbg!(&code.messages);
    // dbg!(&code.services);
    // dbg!(&code.imports);
    // dbg!(&code.counters);
    // dbg!(&code.paths);
    /* let mut file = File::create("src/interface.rs").unwrap();
    file.write_all(preamble.as_bytes())
        .unwrap(); 
    */
    println!("Enum data for size");
    dbg!(&code.enums[0]);
    // dbg!(&code.enumflags[0]);
    dbg!(&code.messages);

    
}
// Things to do 
// 1. Remove vl_message_id in struct - Done
// 2. Use serde_repr for enums of certain types - Done  
// 3. Create generated mod for testing 
// 4. Test all messages from interface.api.json 
// 5. Repeat the strategy for the rest of the core api jsons 
// 6. Do not waste time trying to test similar messages 
// 7. Find largest field in enum 

pub fn gen_structs(structs: &VppJsApiType, file: &mut String, apifile: &VppJsApiFile){
    sizeof_struct(&structs, &apifile);
    file.push_str(&format!("#[derive(Debug, Clone, Serialize, Deserialize)] \n"));
    file.push_str(&format!("pub struct {} {{ \n",camelize_ident(&structs.type_name)));
    for x in 0..structs.fields.len(){
        file.push_str(&format!("\tpub {} : {}, \n", get_ident(&structs.fields[x].name), get_type(&structs.fields[x].ctype)));
    }
    file.push_str("} \n");
}
pub fn gen_union(unions: &VppJsApiType, file: &mut String){
    // maxSizeUnion(&unions);
    file.push_str(&format!("#[derive(Debug, Clone, Serialize, Deserialize)] \n"));
    file.push_str(&format!("union {} {{ \n",unions.type_name));
    for x in 0..unions.fields.len(){
        file.push_str(&format!("\t {} : {}, \n", unions.fields[x].name, get_type(&unions.fields[x].ctype)));
    }
    file.push_str("} \n");
}
pub fn gen_enum(enums: &VppJsApiEnum, file: &mut String){
    file.push_str(&format!("#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] \n"));
    match &enums.info.enumtype {
        Some(len) => file.push_str(&format!("#[repr({})]\n",&len)),
        _ => file.push_str(&format!("#[repr(u32)]\n"))
    }
    file.push_str(&format!("pub enum {} {{ \n", camelize_ident(&enums.name)));
    for x in 0..enums.values.len(){
        file.push_str(&format!("\t {}={}, \n",get_ident(&enums.values[x].name), enums.values[x].value));
    }
    file.push_str("} \n");
}
pub fn gen_alias(alias:&VppJsApiAlias, name: &str, file: &mut String){
    sizeof_alias(&alias);
    file.push_str(&format!("pub type {}=", camelize_ident(&get_ident(&name))));
    // println!("{}", name);
    // print!("{} - ", alias.ctype);
    match alias.length{
        Some(len) => {
            let newtype = get_type(&alias.ctype);
            file.push_str(&format!("[{};{}]; \n",newtype,len));
        },
        _ => file.push_str(&format!("{}; \n", get_type(&alias.ctype))),
    }
    // println!();
}
pub fn gen_messages(messages: &VppJsApiMessage, file: &mut String) {
    file.push_str(&format!("#[derive(Debug, Clone, Serialize, Deserialize)] \n"));
    file.push_str(&format!("pub struct {} {{ \n",camelize_ident(&messages.name)));
    for x in 0..messages.fields.len(){
        if messages.fields[x].name == "_vl_msg_id" {
            // panic!("Something wrong");
        }
        else if messages.fields[x].ctype == "string" {
            match &messages.fields[x].maybe_size {
                Some(cont) => {
                    match cont {
                        VppJsApiFieldSize::Fixed(len) => file.push_str(&format!("\tpub {} : FixedSizeString<U{}>, \n", get_ident(&messages.fields[x].name), len)),
                        VppJsApiFieldSize::Variable(None) => file.push_str(&format!("\tpub {} : VariableSizeString, \n", get_ident(&messages.fields[x].name))),
                        _ => file.push_str(&format!("\tpub {} : , \n", get_ident(&messages.fields[x].name))),
                    }
                }
                _ => file.push_str(&format!("\tpub {} :, \n", get_ident(&messages.fields[x].name)))
            }
            // file.push_str(&format!("\tpub {} : {}, \n", get_ident(&messages.fields[x].name), get_type(&messages.fields[x].ctype)));
        }
        else {
            // print!("{}",messages.fields[x].name);
            file.push_str(&format!("\tpub {} : {}, \n", get_ident(&messages.fields[x].name), get_type(&messages.fields[x].ctype)));
        }
        
    }
    file.push_str("} \n");
    gen_impl_messages(messages, file);
}

pub fn gen_impl_messages(messages: &VppJsApiMessage, file: &mut String){
    file.push_str(&format!("impl {} {{ \n",camelize_ident(&messages.name)));
    file.push_str(&format!("\t pub fn get_message_id() -> String {{ \n"));
    file.push_str(&format!("\t \t String::from(\"{}_{}\") \n",messages.name, messages.info.crc.trim_start_matches("0x")));
    file.push_str(&format!("\t }} \n"));
    file.push_str(&format!("}} \n"));
    
}
