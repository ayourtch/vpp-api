#![allow(dead_code,unused_mut,unused_variables,unused_must_use, non_camel_case_types,unused_imports)]
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

/// This program does something useful, but its author needs to edit this.
/// Else it will be just hanging around forever
#[derive(Debug, Clone, Clap, Serialize, Deserialize)]
#[clap(version = env!("GIT_VERSION"), author = "Andrew Yourtchenko <ayourtch@gmail.com>")]
struct Opts {
    /// Target hostname to do things on
    #[clap(short, long, default_value = "localhost")]
    target_host: String,

    /// Use AF_UNIX socket if this path is mentioned, else use shared mem
    #[clap(short, long)]
    socket_path: Option<String>,

    /// Override options from this yaml/json file
    #[clap(short, long)]
    options_override: Option<String>,

    /// set non-blocking mode for the connection
    #[clap(short, long)]
    nonblocking: bool,

    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn get_encoder() -> impl bincode::config::Options {
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

use vpp_api_transport::afunix;
use vpp_api_transport::shmem;
use vpp_api_transport::VppApiTransport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPRouteDump {
    pub client_index: u32, 
    pub context: u32, 
    pub table: IPTable
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPTable{
    pub table_id: u32, 
    pub is_ip6: bool, 
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPRouteDetails{
    pub context: u32, 
    pub route: IPRoute
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPRoute{
    pub table_id: u32, 
    pub stats_index: u32, 
    pub prefix: Prefix, 
    pub n_paths: u8, 

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPing {
    pub client_index: u32,
    pub context: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPingReply {
    pub context: u32,
    pub retval: i32,
    pub client_index: u32,
    pub vpe_pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddressDump {
    pub client_index: u32, 
    pub context: u32, 
    pub sw_if_index: InterfaceIndex, 
    pub is_ipv6: bool,
}
// 2d033de4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddressDetails{
    pub context: u32, 
    pub sw_if_index: InterfaceIndex,
    pub prefix: Prefix
}
// b1199745


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SWInterfaceAddDelAddress {
    pub client_index: u32,
    pub context: u32,
    pub sw_if_index: InterfaceIndex,
    pub is_add: bool,
    pub del_all: bool,
    pub prefix: Prefix
}
type InterfaceIndex = u32;
// type InterfaceIndex = u32;
// Experimental Test so that encoding doesn't have to happen inside send_recv 
/* impl SWInterfaceAddDelAddress{
    fn EncodeSWInterfaceAddDelAddress(&self){
        // making bytes 
        let enc = get_encoder();
        let mut v = enc.serialize(&self.client_index).unwrap();
        let enc = get_encoder();
        let context = enc.serialize(&self.context).unwrap();
        let enc = get_encoder();
        let sw_if_index = enc.serialize(&self.sw_if_index).unwrap();
        let enc = get_encoder();
        let isadd = enc.serialize(&self.is_add).unwrap();
        let enc = get_encoder();
        let del_all = enc.serialize(&self.del_all).unwrap();
        let enc = get_encoder();
        println!("Address family stores {:?}", &self.prefix.address.af);
        let af = enc.serialize(&self.prefix.address.af).unwrap();
        let enc = get_encoder();
        let un:Vec<u8>; 
        match &self.prefix.address.un {
            AddressUnion::IP4(addr) => un = enc.serialize(addr).unwrap(),
            AddressUnion::IP6(addr) => un = enc.serialize(addr).unwrap()
        } 
        let enc = get_encoder();
        let plen = enc.serialize(&self.prefix.len).unwrap();
        
        dbg!(&af);
        dbg!(&un);
        v.extend_from_slice(&context);
        v.extend_from_slice(&sw_if_index);
        v.extend_from_slice(&isadd);
        v.extend_from_slice(&del_all);
        v.extend_from_slice(&af);
        v.extend_from_slice(&un);
        v.extend_from_slice(&plen);
        dbg!(v);
        let enc = get_encoder();
        let mut v = enc.serialize(&self.prefix.address.un).unwrap();
        dbg!(v);
    }
}*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefix {
    pub address: Address,
    pub len: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub af: AddressFamily ,
    pub un: AddressUnion,
}
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum AddressFamily {
    ADDRESS_IP4 = 0,
    ADDRESS_IP6 = 1,
}
// #[derive(Debug, Clone, Serialize, Deserialize)]
/* pub struct AddressUnion {
    IP4: [u8; 4],
}*/
/* #[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddressUnion{
    IP4([u8;16]), 
    IP6([u8;16]),
} */ 
type AddressUnion = [u8;16];
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SWInterfaceAddDelAddressReply {
    pub context: u32,
    pub retval: i32,
}

/* pub fn test_func_interface() {
    let t = SWInterfaceAddDelAddress{
        sw_if_index: 0,
        context: 0,
        client_index: 0, 
        is_add: true,
        del_all: false,
        prefix: Prefix{
            address: Address{
                af: AddressFamily::ADDRESS_IP4,
                un: AddressUnion::IP4([0x0a,0x0b,0x0c,0x0d])
            },
            len: 10
        },
    };
    t.EncodeSWInterfaceAddDelAddress();
    
} */



fn send_recv_msg<'a, T: Serialize + Deserialize<'a>, TR: Serialize + DeserializeOwned >(
    name: &str,
    m: &T,
    t: &mut dyn VppApiTransport,
    reply_name: &str,
) -> TR {
    let vl_msg_id = t.get_msg_index(name).unwrap();
    let reply_vl_msg_id = t.get_msg_index(reply_name).unwrap();
    let enc = get_encoder();
    let mut v = enc.serialize(&vl_msg_id).unwrap();
    let enc = get_encoder();
    let msg = enc.serialize(&m).unwrap();
    
    v.extend_from_slice(&msg);
    println!("MSG[{} = 0x{:x}]: {:?}", name, vl_msg_id, &v);
    t.write(&v);
    let nres = t.read_one_msg_id_and_msg();
    dbg!(nres);
    loop {
        let res = t.read_one_msg_id_and_msg();
        // dbg!(&res);
        if let Ok((msg_id, data)) = res {
            println!("id: {} data: {:x?}", msg_id, &data);
            if msg_id == reply_vl_msg_id {
                let res = get_encoder()
                    .allow_trailing_bytes()
                    .deserialize::<TR>(&data)
                    .unwrap();
                println!("Next thing will be the reply");
                return res;
            } else {
                println!("Checking the next message for the reply id");
            }
        } else {
            panic!("Result is an error: {:?}", &res);
        }
    }
}

fn send_bulk_msg<'a, T: Serialize + Deserialize<'a>,TR: Serialize + DeserializeOwned + std::fmt::Debug>(
    name: &str,
    m: &T,
    t: &mut dyn VppApiTransport,
    reply_name: &str,
) -> TR {
    let control_ping_id = t.get_msg_index("control_ping_51077d14").unwrap();
    let control_ping_id_reply = t.get_msg_index("control_ping_reply_f6b0b8ca").unwrap();
    let vl_msg_id = t.get_msg_index(name).unwrap();
    let reply_vl_msg_id = t.get_msg_index(reply_name).unwrap();
    let enc = get_encoder();
    let mut v = enc.serialize(&vl_msg_id).unwrap();
    let enc = get_encoder();
    let msg = enc.serialize(&m).unwrap();/////
    let control_ping = ControlPing {
        client_index: t.get_client_index(), 
        context: 0
    };
    let enc = get_encoder();
    let mut c = enc.serialize(&control_ping_id).unwrap();
    let enc = get_encoder();
    let control_ping_message = enc.serialize(&control_ping).unwrap();
    c.extend_from_slice(&control_ping_message);
    v.extend_from_slice(&msg);
    println!("MSG[{} = 0x{:x}]: {:?}", "control ping", control_ping_id, &c);
    println!("MSG[{} = 0x{:x}]: {:?}", name, vl_msg_id, &v);
    // dbg!(&c);
    let mut out: Vec<u8> = vec![];
    t.write(&v); // Dump message 
    // let res = t.read_one_msg_into(&mut out);
    t.write(&c); // Ping message 
    // dbg!(&out);
    dbg!(control_ping_id_reply);
    let mut out: Vec<TR>;
    let mut count = 0;
    // t.write(&c);
    loop {
        println!("Reached loop");
        let res = t.read_one_msg_id_and_msg();
        // dbg!(&out);
        if let Ok((msg_id, data)) = res {
            println!("id: {} data: {:x?}", msg_id, &data);
            if msg_id == control_ping_id_reply{
                /*let res = get_encoder()
                    .allow_trailing_bytes()
                    .deserialize::<TR>(&data)
                    .unwrap();
                println!("Next thing will be the reply");
                return res;*/ 
                // break;
                // break
               // return out;
               // break;
               // return res;
            } 
            if msg_id == reply_vl_msg_id{
                println!("Received the intended message");
                let res = get_encoder()
                    .allow_trailing_bytes()
                    .deserialize::<TR>(&data)
                    .unwrap();
                println!("Next thing will be the reply");
                return res;
                //return res; 

            }
            else {
                println!("Checking the next message for the reply id");
            }
            
        } else {
            panic!("Result is an error: {:?}", &res);
        }
    }
}


fn main() {
    let opts: Opts = Opts::parse();

    // allow to load the options, so far there is no good built-in way
    let opts = if let Some(fname) = &opts.options_override {
        if let Ok(data) = std::fs::read_to_string(&fname) {
            let res = serde_json::from_str(&data);
            if res.is_ok() {
                res.unwrap()
            } else {
                serde_yaml::from_str(&data).unwrap()
            }
        } else {
            opts
        }
    } else {
        opts
    };

    if opts.verbose > 4 {
        let data = serde_json::to_string_pretty(&opts).unwrap();
        println!("{}", data);
        println!("===========");
        let data = serde_yaml::to_string(&opts).unwrap();
        println!("{}", data);
    }

    println!("Hello, here is your options: {:#?}", &opts);
    println!("Here is your interface reply");
    // test_func_interface();
    // test_func();
    // let mut t = shmem::Transport::new();
    // let mut t = afunix::Transport::new("/tmp/api.sock");
    let mut t: Box<dyn VppApiTransport> = if let Some(afunix_path) = &opts.socket_path {
        Box::new(afunix::Transport::new(&afunix_path))
    } else {
        Box::new(shmem::Transport::new())
    };

    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    t.set_nonblocking(opts.nonblocking);

    /* let create_interface_reply: SWInterfaceAddDelAddressReply = send_recv_msg(
        "sw_interface_add_del_address_5803d5c4",
        &SWInterfaceAddDelAddress {
            client_index: t.get_client_index(),
            context: 0,
            sw_if_index: 1,
            is_add: true,
            del_all: false,
            af: 0, 
            un: [0xa,0xa,1,2,7,0x7a,0xb,0xc,0xd,0xf,8,9,5,6,10,10], 
            len: 24         
        },
        &mut *t,
        "sw_interface_add_del_address_reply_e8d4e804"
    );
    // [0xa,0xa,1,2,7,0x7a,0xb,0xc,0xd,0xf,8,9,5,6,10,10]
    // 10.10.1.2/24
    println!("Create Interface Reply: {:#?}", &create_interface_reply);
    */ 
     let ipaddress:IpAddressDetails  = send_bulk_msg(
        "ip_address_dump_2d033de4",
        &IpAddressDump{
            client_index: t.get_client_index(), 
            context: 0,
            sw_if_index: 1, 
            is_ipv6: false
        },
        &mut *t,
        "ip_address_details_b1199745"
    );
    // [0xa,0xa,1,2,7,0x7a,0xb,0xc,0xd,0xf,8,9,5,6,10,10]
    // 10.10.1.2/24
    println!("Show IP Address Reply: {:#?}", &ipaddress);
    
  

    // t.control_ping();
    //
    // bench(&mut *t);

    std::thread::sleep(std::time::Duration::from_secs(1));
    t.disconnect();

    // std::thread::sleep(std::time::Duration::from_secs(1));
}
