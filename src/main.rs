use clap::Clap;
use serde::{Deserialize, Serialize};
use std::string::ToString;
extern crate strum;
#[macro_use]
extern crate strum_macros;
use env_logger;
use linked_hash_map::LinkedHashMap;
mod message; 
mod types;
mod alias;
mod services;
mod enums;
mod parser_helper;
mod file_schema;
mod code_gen;
mod basetypes;
mod interface;
// mod interface;
use crate::parser_helper::*;
use crate::message::*;
use crate::file_schema::VppJsApiFile;
use crate::types::*;
use crate::code_gen::gen_code;
use crate::interface::*;
// use crate::interface::*;

#[derive(Clap, Debug, Clone, Serialize, Deserialize, EnumString, Display)]
pub enum OptParseType {
    File,
    Tree,
    ApiType,
    ApiMessage,
}

/// Ingest the VPP API JSON definition file and output the Rust code
#[clap(version = "0.1", author = "Andrew Yourtchenko <ayourtch@gmail.com>")]
#[derive(Clap, Debug, Clone, Serialize, Deserialize)]
pub struct Opts {
    /// Input file name
    #[clap(short, long)]
    pub in_file: String,

    /// output file name
    #[clap(short, long, default_value = "dummy.rs")]
    pub out_file: String,

    /// parse type for the operation: Tree, File, ApiMessage or ApiType
    #[clap(short, long, default_value = "File")]
    pub parse_type: OptParseType,

    /// Print message names
    #[clap(long)]
    pub print_message_names: bool,

    /// Generate the code
    #[clap(long)]
    pub generate_code: bool,

    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
}




fn main() {
    env_logger::init();
    let opts: Opts = Opts::parse();
    log::info!("Starting file {}", &opts.in_file);

    if let Ok(data) = std::fs::read_to_string(&opts.in_file) {
        match opts.parse_type {
            OptParseType::Tree => {
                panic!("Can't parse a tree out of file!");
            }
            OptParseType::File => {
                let desc = VppJsApiFile::from_str(&data).unwrap();
                eprintln!(
                    "File: {} version: {} services: {} types: {} messages: {} aliases: {} imports: {} enums: {} unions: {}",
                    &opts.in_file,
                    &desc.vl_api_version,
                    desc.services.len(),
                    desc.types.len(),
                    desc.messages.len(),
                    desc.aliases.len(),
                    desc.imports.len(),
                    desc.enums.len(),
                    desc.unions.len()
                );
                if opts.verbose > 1 {
                    println!("Dump File: {:#?}", &desc);
                }
                let data = serde_json::to_string_pretty(&desc).unwrap();
                // println!("{}", &data);
                gen_code(&desc);
            }
            OptParseType::ApiType => {
                let desc: VppJsApiType = serde_json::from_str(&data).unwrap();
                println!("Dump Type: {:#?}", &desc);
            }
            OptParseType::ApiMessage => {
                let desc: VppJsApiMessage = serde_json::from_str(&data).unwrap();
                println!("Dump: {:#?}", &desc);
            }
        }
    } else {
        match opts.parse_type {
            OptParseType::Tree => {
                // it was a directory tree, descend downwards...
                let mut api_files: LinkedHashMap<String, VppJsApiFile> = LinkedHashMap::new();
                parse_api_tree(&opts, &opts.in_file, &mut api_files);
                println!("// Loaded {} API definition files", api_files.len());
                if opts.print_message_names {
                    for (_name, f) in &api_files {
                        for m in &f.messages {
                            let crc = &m.info.crc.strip_prefix("0x").unwrap();
                            println!("{}_{}", &m.name, &crc);
                        }
                    }
                }
                if opts.generate_code {
                    generate_code(&opts, &api_files);
                }
            }
            e => {
                panic!("inappropriate parse type {:?} for inexistent file", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_data_path() -> PathBuf {
        let mut path = PathBuf::from(file!());
        path.pop();
        path.pop();
        path.pop();
        path.push("testdata/vpp/");
        path
    }

    fn parse_api_tree_with_verify(root: &str, map: &mut LinkedHashMap<String, VppJsApiFile>) {
        use std::fs;
        for entry in fs::read_dir(root).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            let metadata = fs::metadata(&path).unwrap();
            if metadata.is_file() {
                let res = std::fs::read_to_string(&path);
                if let Ok(data) = res {
                    let desc = VppJsApiFile::from_str(&data);
                    if let Ok(d) = desc {
                        VppJsApiFile::verify_data(&data, &d);
                        map.insert(path.to_str().unwrap().to_string(), d);
                    } else {
                        eprintln!("Error loading {:?}: {:?}", &path, &desc);
                    }
                } else {
                    eprintln!("Error reading {:?}: {:?}", &path, &res);
                }
            }
            if metadata.is_dir() && entry.file_name() != "." && entry.file_name() != ".." {
                parse_api_tree_with_verify(&path.to_str().unwrap(), map);
            }
        }
    }

    #[test]
    fn test_tree() {
        let mut api_files: LinkedHashMap<String, VppJsApiFile> = LinkedHashMap::new();
        parse_api_tree_with_verify(get_test_data_path().to_str().unwrap(), &mut api_files);

        assert_eq!(123, api_files.len());
    }
}
