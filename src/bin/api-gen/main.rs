#![allow(
    dead_code,
    unused_mut,
    unused_variables,
    unused_must_use,
    non_camel_case_types,
    unused_imports,
    non_snake_case
)]
use clap::Clap;
use std::string::ToString;
extern crate strum;
#[macro_use]
extern crate strum_macros;
use env_logger;
use linked_hash_map::LinkedHashMap;
mod alias;
mod basetypes;
mod code_gen;
mod enums;
mod file_schema;
mod message;
mod parser_helper;
mod services;
mod types;
use crate::code_gen::{create_cargo_toml, gen_code, generate_lib_file, copy_file_with_fixup};
use crate::file_schema::VppJsApiFile;
use crate::message::*;
use crate::parser_helper::*;
use crate::types::*;
use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::io::{Read, Write};
use std::ops::Add;
use std::time::{Duration, SystemTime};
use vpp_api_encoding::typ::*;

#[derive(Clap, Debug, Clone, Serialize, Deserialize, EnumString, Display)]
pub enum OptParseType {
    File,
    Tree,
    ApiType,
    ApiMessage,
}

/// Ingest the VPP API JSON definition file and output the Rust code
#[clap(version = "1.0", author = "Andrew Yourtchenko <ayourtch@gmail.com>")]
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

    /// Package name for the generated package
    #[clap(long, default_value = "someVPP")]
    pub package_name: String,

    /// Print message names
    #[clap(long)]
    pub print_message_names: bool,

    /// Generate the bindings within the directory
    #[clap(long)]
    pub create_binding: bool,

    /// Generate the package for the binding
    #[clap(long)]
    pub create_package: bool,

    /// Generate the code
    #[clap(long)]
    pub generate_code: bool,

    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
}
fn merge(mut arr: Vec<ImportsFiles>, left: usize, mid: usize, right: usize) -> Vec<ImportsFiles> {
    let n1 = mid - left;
    let n2 = right - mid;
    let mut L1 = arr.clone();
    let mut R1 = arr.clone();
    let L = &L1[left..mid];
    let R = &R1[mid..right];
    /* Merge the temp arrays back into arr[l..r]*/
    let mut i = 0; // Initial index of first subarray
    let mut j = 0; // Initial index of second subarray
    let mut k = left; // Initial index of merged subarray
    while i < n1 && j < n2 {
        if L[i].file.imports.len() < R[j].file.imports.len() {
            arr[k] = L[i].clone();
            i = i + 1;
        } else {
            arr[k] = R[j].clone();
            j = j + 1;
        }
        k = k + 1;
    }
    while i < n1 {
        arr[k] = L[i].clone();
        i = i + 1;
        k = k + 1;
    }
    /* Copy the remaining elements of R[], if there
    are any */
    while j < n2 {
        arr[k] = R[j].clone();
        j = j + 1;
        k = k + 1;
    }
    arr
}
// Performing Merge Sort According to import lenght
fn merge_sort(mut arr: Vec<ImportsFiles>, left: usize, right: usize) -> Vec<ImportsFiles> {
    if right - 1 > left {
        let mid = left + (right - left) / 2;
        arr = merge_sort(arr, left, mid);
        arr = merge_sort(arr, mid, right);
        arr = merge(arr, left, mid, right);
    }
    arr
}
#[derive(Debug, Clone)]
pub struct ImportsFiles {
    name: String,
    file: Box<VppJsApiFile>,
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
                let mut api_definition: Vec<(String, String)> = vec![];
                gen_code(&desc, "generated2.", &mut api_definition, "test");
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
                    for (name, f) in &api_files {
                        println!("{}", name);
                        for m in &f.messages {
                            let crc = &m.info.crc.strip_prefix("0x").unwrap();
                            // println!("{}_{}", &m.name, &crc);
                        }
                    }
                }
                if opts.generate_code {
                    let mut api_definition: Vec<(String, String)> = vec![];
                    for (name, f) in &api_files {
                        gen_code(
                            f,
                            name.trim_start_matches("testdata/vpp/api")
                                .trim_end_matches("json"),
                            &mut api_definition,
                            "test",
                        );
                    }
                }
                if opts.create_binding {
                    let mut import_collection: Vec<ImportsFiles> = vec![];
                    // Searching for types
                    for (name, f) in api_files.clone() {
                        if name.ends_with("_types.api.json") {
                            import_collection.push(ImportsFiles {
                                name: name.to_string(),
                                file: Box::new(f),
                            })
                        }
                    }
                    let mut api_definition: Vec<(String, String)> = vec![];
                    import_collection =
                        merge_sort(import_collection.clone(), 0, import_collection.len());
                    for x in import_collection {
                        println!("{}-{}", x.name, x.file.imports.len());
                        gen_code(&x.file, &x.name, &mut api_definition, "test");
                    }
                    // Searching for non types
                    for (name, f) in api_files.clone() {
                        if !name.ends_with("_types.api.json") {
                            gen_code(&f, &name, &mut api_definition, "test");
                        }
                    }
                }
                if opts.create_package {
                    // println!("{}", opts.package_name);
                    let mut api_definition: Vec<(String, String)> = vec![];
                    println!("Do whatever you need to hear with creating package");
                    fs::create_dir(&format!(".././{}", opts.package_name)).unwrap();
                    fs::create_dir(&format!(".././{}/src", opts.package_name)).unwrap();
                    fs::create_dir(&format!(".././{}/tests", opts.package_name)).unwrap();
                    fs::create_dir(&format!(".././{}/examples", opts.package_name)).unwrap();
                    fs::File::create(&format!(".././{}/src/reqrecv.rs", opts.package_name)).unwrap();
                    fs::copy("./src/reqrecv.rs", &format!(".././{}/src/reqrecv.rs", opts.package_name)).unwrap();
                    generate_lib_file(&api_files, &opts.package_name);
                    create_cargo_toml(&opts.package_name);
                    copy_file_with_fixup("./tests/interface-test.rs", &opts.package_name, "tests/interface_test.rs");
                    copy_file_with_fixup("./examples/vhost-example.rs", &opts.package_name, "examples/progressive-vpp.rs");

                    let mut import_collection: Vec<ImportsFiles> = vec![];
                    for (name, f) in api_files.clone() {
                        if name.ends_with("_types.api.json") {
                            import_collection.push(ImportsFiles {
                                name: name.to_string(),
                                file: Box::new(f),
                            })
                        }
                    }
                    import_collection =
                        merge_sort(import_collection.clone(), 0, import_collection.len());
                    for x in import_collection {
                        gen_code(&x.file, &x.name, &mut api_definition, &opts.package_name);
                    }
                    for (name, f) in api_files.clone() {
                        if !name.ends_with("_types.api.json") {
                            gen_code(&f, &name, &mut api_definition, &opts.package_name);
                        }
                    }
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
    use bincode::Options;
    use clap::Clap;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_repr::{Deserialize_repr, Serialize_repr};
    use std::collections::HashMap;
    use std::convert::TryInto;
    use std::io::{Read, Write};
    use std::ops::Add;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};
    use vpp_api_encoding::typ::*;

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

    /* #[test]
    fn test_tree() {
        let mut api_files: LinkedHashMap<String, VppJsApiFile> = LinkedHashMap::new();
        parse_api_tree_with_verify(get_test_data_path().to_str().unwrap(), &mut api_files);

        assert_eq!(123, api_files.len());
    }*/
    /* #[test]
    fn test_ip_address_dump() {
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
        // println!("Connect result: {:?}", t.connect("api-test", None, 256));
        // t.get_msg_index("sw_interface_add_del_address_5803d5c4").unwrap();
        t.set_nonblocking(false);
        let create_interface_reply: sw_interface_add_del_address_reply = send_recv_msg(
            "sw_interface_add_del_address_5803d5c4",
            &sw_interface_add_del_address {
                client_index: t.get_client_index(),
                context: 0,
                sw_if_index: 0,
                is_add: true,
                del_all: false,
                prefix: address_with_prefix{
                    address: Address {
                        af: address_family::ADDRESS_IP4,
                        un: [0xa,0xa,1,2,7,0x7a,0xb,0xc,0xd,0xf,8,9,5,6,10,10],
                    },
                    len: 24,
                }
            },
            &mut *t,
            &sw_interface_add_del_address_reply::get_message_name_and_crc()
        );
        assert_eq!(create_interface_reply.context, 0);
        t.disconnect();
    }*/
    /* #[test]
    fn test_transport_connection(){
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
        // dbg!(t.connect("api-test", None, 256));
        let check = t.connect("api-test", None, 256);
        assert_eq!(check,0);
        t.disconnect();
    }*/
    /* #[test]
    fn test_transport_get_msg_indx(){
        let mut t = shmem::Transport::new();
        // dbg!(t.connect("api-test", None, 256));
        // let check = t.connect("api-test", None, 256);
        t.set_nonblocking(false);
        let vl_msg_id = t.get_msg_index("control_ping_51077d14").unwrap();
        assert_ne!(vl_msg_id,0);
        // std::thread::sleep(std::time::Duration::from_secs(1));
        t.disconnect();
    }*/
}
