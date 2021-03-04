use bincode::Options;
use clap::Clap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::time::{Duration, SystemTime};
use vpp_api_encoding::typ::*;
use vpp_api_transport::*;

use typenum::{U10, U256, U32, U64};

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
pub struct TestAPI {
    id: i32,
    foo: FixedSizeString<U10>,
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
pub struct CliInband {
    pub client_index: u32,
    pub context: u32,
    pub cmd: VariableSizeString,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliInbandReply {
    pub context: u32,
    pub retval: i32,
    pub reply: VariableSizeString,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowThreads {
    pub client_index: u32,
    pub context: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadData {
    pub id: u32,
    pub name: FixedSizeString<U64>,
    pub r#type: FixedSizeString<U64>,
    pub pid: u32,
    pub cpu_id: u32,
    pub core: u32,
    pub cpu_socket: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowThreadsReply {
    pub context: u32,
    pub retval: i32,
    pub count: u32,
    pub thread_data: VariableSizeArray<ThreadData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetF64IncrementByOne {
    pub client_index: u32,
    pub context: u32,
    pub f64_value: F64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetF64IncrementByOneReply {
    pub context: u32,
    pub retval: u32,
    pub f64_value: F64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowVersion {
    pub client_index: u32,
    pub context: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowVersionReply {
    pub context: u32,
    pub retval: i32,
    pub program: FixedSizeString<U32>,
    pub version: FixedSizeString<U32>,
    pub build_date: FixedSizeString<U32>,
    pub build_directory: FixedSizeString<U256>,
}

pub fn test_func() {
    let t = CliInband {
        client_index: 0xaaaabbbb,
        context: 0xccccdddd,
        cmd: "testng123".try_into().unwrap(),
    };
    println!("t: {:#x?}", &t);
}

fn bench(t: &mut dyn VppApiTransport) {
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};

    let now = SystemTime::now();

    let count = 100000;
    println!("Starting {} requests", count);

    for i in 1..count {
        let now = SystemTime::now();
        let s = t.run_cli_inband("show interface");
        // t.control_ping();

        // println!("res = {:?}", &s);
        // println!("{:?}", now.elapsed());
    }

    match now.elapsed() {
        Ok(elapsed) => {
            // it prints '2'
            println!(
                "{} : {}/sec",
                elapsed.as_secs_f64(),
                (count as f64) / elapsed.as_secs_f64()
            );
        }
        Err(e) => {
            // an error occurred!
            println!("Error: {:?}", e);
        }
    }
}

fn send_msg<'a, T: Serialize + Deserialize<'a>>(name: &str, m: &T, t: &mut dyn VppApiTransport) {
    let vl_msg_id = t.get_msg_index(name).unwrap();
    let enc = get_encoder();
    let mut v = enc.serialize(&vl_msg_id).unwrap();
    let enc = get_encoder();
    let msg = enc.serialize(&m).unwrap();
    v.extend_from_slice(&msg);
    println!("MSG[{} = 0x{:x}]: {:?}", name, vl_msg_id, &v);
    t.write(&v);
}

fn send_recv_msg<'a, T: Serialize + Deserialize<'a>, TR: Serialize + DeserializeOwned>(
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

    loop {
        let res = t.read_one_msg_id_and_msg();
        if let Ok((msg_id, data)) = res {
            println!("id: {} data: {:x?}", msg_id, &data);
            if msg_id == reply_vl_msg_id {
                let res = get_encoder()
                    .allow_trailing_bytes()
                    .deserialize::<TR>(&data)
                    .unwrap();
                return res;
            }
        } else {
            panic!("Result is an error: {:?}", &res);
        }
    }
}

fn deser3_test() {
    let strep = VariableSizeArray::<u8>(vec![1, 2, 1, 3, 4, 5]);

    println!("structure: {:x?}", &strep);
    let enc = get_encoder();
    let mut v = enc.serialize(&strep).unwrap();
    println!("converted to bin: {:x?}", &v);

    let r: VariableSizeArray<u8> = get_encoder()
        .allow_trailing_bytes()
        .deserialize(&v)
        .unwrap();
    println!("from bin: {:#x?}", &r);

    let data = serde_json::to_string(&r).unwrap();
    println!("JSON: {}", data);
}
fn deser2_test() {
    // deser3_test();

    let td = ThreadData {
        id: 0,
        name: "some name".try_into().unwrap(),
        r#type: "worker".try_into().unwrap(),
        pid: 1234,
        cpu_id: 0,
        core: 0,
        cpu_socket: 42,
    };
    let strep = VariableSizeArray(vec![td.clone(), td.clone(), td.clone()]);

    println!("structure: {:x?}", &strep);
    let enc = get_encoder();
    let mut v = enc.serialize(&strep).unwrap();
    println!("converted to bin: {:x?}", &v);

    let r: VariableSizeArray<ThreadData> = get_encoder()
        .allow_trailing_bytes()
        .deserialize(&v)
        .unwrap();
    println!("from bin: {:x?}", &r);

    let data = serde_json::to_string(&r).unwrap();
    println!("JSON: {}", data);
}

fn deser_test() {
    let td = ThreadData {
        id: 0,
        name: "some name".try_into().unwrap(),
        r#type: "worker".try_into().unwrap(),
        pid: 1234,
        cpu_id: 0,
        core: 0,
        cpu_socket: 42,
    };
    let strep = ShowThreadsReply {
        context: 0x42424242,
        retval: 42,
        count: 1,
        thread_data: VariableSizeArray(vec![td]),
    };

    println!("structure: {:x?}", &strep);
    let enc = get_encoder();
    let mut v = enc.serialize(&strep).unwrap();
    println!("converted to bin: {:x?}", &v);

    let r: ShowThreadsReply = get_encoder()
        .allow_trailing_bytes()
        .deserialize(&v)
        .unwrap();
    println!("from bin: {:#x?}", &r);

    let data = serde_json::to_string(&r).unwrap();
    println!("JSON: {}", data);
}

fn deser_cli_inband_test() {
    let strep = CliInbandReply {
        context: 0x42424242,
        retval: 42,
        reply: "testing123".try_into().unwrap(),
    };

    println!("structure: {:x?}", &strep);
    let enc = get_encoder();
    let mut v = enc.serialize(&strep).unwrap();
    println!("converted to bin: {:x?}", &v);

    let r: CliInbandReply = get_encoder()
        .allow_trailing_bytes()
        .deserialize(&v)
        .unwrap();
    println!("from bin: {:#x?}", &r);

    let data = serde_json::to_string(&r).unwrap();
    println!("JSON: {}", data);
}

fn main() {
    deser_test();
    deser_cli_inband_test();
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

    // println!("Hello, here is your options: {:#?}", &opts);
    test_func();
    // let mut t = shmem::Transport::new();
    // let mut t = afunix::Transport::new("/tmp/api.sock");
    let mut t: Box<dyn VppApiTransport> = if let Some(afunix_path) = &opts.socket_path {
        Box::new(afunix::Transport::new(&afunix_path))
    } else {
        Box::new(shmem::Transport::new())
    };

    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    t.set_nonblocking(opts.nonblocking);

    let ping_reply: ControlPingReply = send_recv_msg(
        "control_ping_51077d14",
        &ControlPing {
            client_index: t.get_client_index(),
            context: 0,
        },
        &mut *t,
        "control_ping_reply_f6b0b8ca",
    );
    println!("Control ping reply: {:#?}", &ping_reply);

    let cli_reply: CliInbandReply = send_recv_msg(
        "cli_inband_f8377302",
        &CliInband {
            client_index: t.get_client_index(),
            context: 0,
            cmd: "show version".try_into().unwrap(),
        },
        &mut *t,
        "cli_inband_reply_05879051",
    );
    println!("cli reply: {:#?}", &cli_reply);

    let show_threads_reply: ShowThreadsReply = send_recv_msg(
        "show_threads_51077d14",
        &ShowThreads {
            client_index: t.get_client_index(),
            context: 0,
        },
        &mut *t,
        "show_threads_reply_efd78e83",
    );
    println!("threads reply: {:#?}", &show_threads_reply);

    let mut f64_inc_reply: GetF64IncrementByOneReply = send_recv_msg(
        "get_f64_increment_by_one_b64f027e",
        &GetF64IncrementByOne {
            client_index: t.get_client_index(),
            context: 0,
            f64_value: F64(1.0),
        },
        &mut *t,
        "get_f64_increment_by_one_reply_d25dbaa3",
    );
    println!("{:?}", &f64_inc_reply);

    // t.control_ping();
    //
    // bench(&mut *t);

    std::thread::sleep(std::time::Duration::from_secs(1));
    t.disconnect();

    // std::thread::sleep(std::time::Duration::from_secs(1));
}
