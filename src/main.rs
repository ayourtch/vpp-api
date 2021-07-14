use clap::Clap;
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
mod MessageFunctions;
use crate::MessageFunctions::*;
use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::ops::Add;
use std::time::{Duration, SystemTime};
use vpp_api_encoding::typ::*;
use vpp_api_transport::*;
use serde_repr::{Serialize_repr, Deserialize_repr};
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
    use super::interface::*;
    use super::MessageFunctions::*;
    use std::path::PathBuf;
    use bincode::Options;
    use clap::Clap;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use std::collections::HashMap;
    use std::convert::TryInto;
    use std::io::{Read, Write};
    use std::ops::Add;
    use std::time::{Duration, SystemTime};
    use vpp_api_encoding::typ::*;
    use vpp_api_transport::*;
    use serde_repr::{Serialize_repr, Deserialize_repr};
    use vpp_api_transport::afunix;
    use vpp_api_transport::shmem;
    use vpp_api_transport::VppApiTransport;

    

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
            &sw_interface_add_del_address_reply::get_message_id()
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
#[cfg(test)]
mod tests_vpp {
    use std::mem::drop;
    use super::*;
    use bincode::Options;
    use clap::Clap;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use std::collections::HashMap;
    use std::convert::TryInto;
    use std::io::{Read, Write};
    use std::ops::Add;
    use std::time::{Duration, SystemTime};
    use vpp_api_encoding::typ::*;
    use vpp_api_transport::*;
    use serde_repr::{Serialize_repr, Deserialize_repr};
    
    use typenum::{U10, U24, U256, U32, U64};
    
    use vpp_api_transport::afunix;
    use vpp_api_transport::shmem;
    use vpp_api_transport::VppApiTransport;
    
    
    
    fn get_encoder() -> impl bincode::config::Options {
        bincode::DefaultOptions::new()
            .with_big_endian()
            .with_fixint_encoding()
    }

    #[test]
    fn test_vpp_functions() {
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));
        println!("Connect result: {:?}", t.connect("api-test", None, 256));
        // dbg!(t.connect("api-test", None, 256));
        let vl_msg_id = t.get_msg_index("control_ping_51077d14").unwrap();
        assert_eq!(vl_msg_id,571);
    }
    #[test]
    fn test_sw_interface_add_del_address() {
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));    
        println!("Connect result: {:?}", t.connect("api-test", None, 256));
        dbg!(t.connect("api-test", None, 256));
        t.set_nonblocking(false);

        let create_interface: SwInterfaceAddDelAddressReply = send_recv_msg(
            &SwInterfaceAddDelAddress::get_message_id(), 
            &SwInterfaceAddDelAddress{
                client_index: t.get_client_index(),
                context: 0, 
                is_add: true,
                del_all: false,
                sw_if_index: 0,
                prefix: AddressWithPrefix{
                    address: Address{
                        af: AddressFamily::ADDRESS_IP4,
                        un: [0xa,0xa,1,2,7,0x7a,0xb,0xc,0xd,0xf,8,9,5,6,10,10],
                    },
                    len:24
                }
                
            }, 
            &mut *t, 
            &SwInterfaceAddDelAddressReply::get_message_id());
       
        assert_eq!(create_interface.context, 0);
        t.disconnect();
        // drop(t);
        // share_vpp(t);
        // std::thread::sleep(std::time::Duration::from_secs(10));
    
    }
    #[test]
    fn test_sw_interface_set_flags() {
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));    
        println!("Connect result: {:?}", t.connect("api-test", None, 256));
        dbg!(t.connect("api-test", None, 256));
        t.set_nonblocking(false);

        let create_interface: SwInterfaceSetFlagsReply = send_recv_msg(
            &SwInterfaceSetFlags::get_message_id(), 
            &SwInterfaceSetFlags{
                client_index: t.get_client_index(),
                context: 0, 
                sw_if_index: 0,
                flags: IfStatusFlags::IF_STATUS_API_FLAG_LINK_UP
            }, 
            &mut *t, 
            &SwInterfaceSetFlagsReply::get_message_id());
       
        assert_eq!(create_interface.context, 0);
        t.disconnect();
        // drop(t);
        // share_vpp(t);
        // std::thread::sleep(std::time::Duration::from_secs(10));
    
    }
    #[test]
    fn test_sw_interface_set_promisc() {
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));    
        println!("Connect result: {:?}", t.connect("api-test", None, 256));
        // dbg!(t.connect("api-test", None, 256));
        t.set_nonblocking(false);

        let create_interface: SwInterfaceSetPromiscReply = send_recv_msg(
            &SwInterfaceSetPromisc::get_message_id(), 
            &SwInterfaceSetPromisc{
                client_index: t.get_client_index(),
                context: 0, 
                sw_if_index: 0,
                promisc_on: false,
            }, 
            &mut *t, 
            &SwInterfaceSetPromiscReply::get_message_id());
       
        assert_eq!(create_interface.context, 0);
        t.disconnect();
        // drop(t);
        // share_vpp(t);
        // std::thread::sleep(std::time::Duration::from_secs(10));
    
    }
    #[test]
    fn test_hw_interface_set_mtu() {
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));    
        println!("Connect result: {:?}", t.connect("api-test", None, 256));
        dbg!(t.connect("api-test", None, 256));
        t.set_nonblocking(false);
        // let vl_msg_id = t.get_msg_index(&HwInterfaceSetMtu::get_message_id()).unwrap();

        let create_interface: HwInterfaceSetMtuReply = send_recv_msg(
            &HwInterfaceSetMtu::get_message_id(), 
            &HwInterfaceSetMtu{
                client_index: t.get_client_index(),
                context: 0, 
                sw_if_index: 0,
                mtu: 50
            }, 
            &mut *t, 
            &HwInterfaceSetMtuReply::get_message_id());
       
        assert_eq!(create_interface.context, 0);
        t.disconnect();
        // drop(t);
        // share_vpp(t);
        // std::thread::sleep(std::time::Duration::from_secs(10));
    
    }
    #[test]
    fn test_sw_interface_set_mtu() {
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));    
        println!("Connect result: {:?}", t.connect("api-test", None, 256));
        dbg!(t.connect("api-test", None, 256));
        t.set_nonblocking(false);
        // let vl_msg_id = t.get_msg_index(&SwInterfaceSetMtu::get_message_id()).unwrap();

        let create_interface: SwInterfaceSetMtuReply = send_recv_msg(
            &SwInterfaceSetMtu::get_message_id(), 
            &SwInterfaceSetMtu{
                client_index: t.get_client_index(),
                context: 0, 
                sw_if_index: 0,
                mtu: 50
            }, 
            &mut *t, 
            &SwInterfaceSetMtuReply::get_message_id());
       
        assert_eq!(create_interface.context, 0);
        t.disconnect();
        // drop(t);
        // share_vpp(t);
        // std::thread::sleep(std::time::Duration::from_secs(10));
    
    }
    #[test]
    fn test_sw_interface_set_ip_directed_broadcast() {
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));    
        println!("Connect result: {:?}", t.connect("api-test", None, 256));
        dbg!(t.connect("api-test", None, 256));
        t.set_nonblocking(false);
        // let vl_msg_id = t.get_msg_index(&SwInterfaceSetIpDirectedBroadcast::get_message_id()).unwrap();

        let create_interface: SwInterfaceSetIpDirectedBroadcastReply = send_recv_msg(
            &SwInterfaceSetIpDirectedBroadcast::get_message_id(), 
            &SwInterfaceSetIpDirectedBroadcast{
                client_index: t.get_client_index(),
                context: 0, 
                sw_if_index: 0,
                enable: true,
            }, 
            &mut *t, 
            &SwInterfaceSetIpDirectedBroadcastReply::get_message_id());
       
        assert_eq!(create_interface.context, 0);
        t.disconnect();
        // drop(t);
        // share_vpp(t);
        // std::thread::sleep(std::time::Duration::from_secs(10));
    
    }
    #[test]
    fn test_want_interface_events() {
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));    
        println!("Connect result: {:?}", t.connect("api-test", None, 256));
        dbg!(t.connect("api-test", None, 256));
        t.set_nonblocking(false);
        // let vl_msg_id = t.get_msg_index(&WantInterfaceEvents::get_message_id()).unwrap();

        let create_interface: WantInterfaceEventsReply = send_recv_msg(
            &WantInterfaceEvents::get_message_id(), 
            &WantInterfaceEvents{
                client_index: t.get_client_index(),
                context: 0, 
                enable_disable: 32,
                pid: 32
            }, 
            &mut *t, 
            &WantInterfaceEventsReply::get_message_id());
       
        assert_eq!(create_interface.context, 0);
        t.disconnect();
        // drop(t);
        // share_vpp(t);
        // std::thread::sleep(std::time::Duration::from_secs(10));
    
    }
    #[test]
    fn test_sw_interface_address_replace_begin() {
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));    
        println!("Connect result: {:?}", t.connect("api-test", None, 256));
        dbg!(t.connect("api-test", None, 256));
        t.set_nonblocking(false);
        // let vl_msg_id = t.get_msg_index(&WantInterfaceEvents::get_message_id()).unwrap();

        let create_interface: SwInterfaceAddressReplaceBeginReply = send_recv_msg(
            &SwInterfaceAddressReplaceBegin::get_message_id(), 
            &SwInterfaceAddressReplaceBegin{
                client_index: t.get_client_index(),
                context: 0, 
            }, 
            &mut *t, 
            &SwInterfaceAddressReplaceBeginReply::get_message_id());
       
        assert_eq!(create_interface.context, 0);
        t.disconnect();
        // drop(t);
        // share_vpp(t);
        // std::thread::sleep(std::time::Duration::from_secs(10));
    
    }
    #[test]
    fn test_sw_interface_address_replace_end() {
        let mut t: Box<dyn VppApiTransport> = Box::new(afunix::Transport::new("/run/vpp/api.sock"));    
        println!("Connect result: {:?}", t.connect("api-test", None, 256));
        dbg!(t.connect("api-test", None, 256));
        t.set_nonblocking(false);
        // let vl_msg_id = t.get_msg_index(&WantInterfaceEvents::get_message_id()).unwrap();

        let create_interface: SwInterfaceAddressReplaceEndReply = send_recv_msg(
            &SwInterfaceAddressReplaceEnd::get_message_id(), 
            &SwInterfaceAddressReplaceEnd{
                client_index: t.get_client_index(),
                context: 0, 
            }, 
            &mut *t, 
            &SwInterfaceAddressReplaceEndReply::get_message_id());
       
        assert_eq!(create_interface.context, 0);
        t.disconnect();
        // drop(t);
        // share_vpp(t);
        // std::thread::sleep(std::time::Duration::from_secs(10));
    
    }

}



