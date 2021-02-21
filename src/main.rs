use bincode::Options;
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::time::{Duration, SystemTime};
use vpp_api_encoding::typ::*;
use vpp_api_transport::*;

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

fn bench(t: &mut dyn VppApiTransport) {
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};

    let now = SystemTime::now();

    let count = 1000000;
    println!("Starting {} requests", count);

    for i in 1..count {
        let s = t.run_cli_inband("show interface");
        // t.control_ping();
        // println!("{:?}", &s);
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

    // println!("Hello, here is your options: {:#?}", &opts);
    test_func();
    let mut t = shmem::Transport::new();
    // let mut t = afunix::Transport::new("/tmp/api.sock");
    /*
    let mut t: Box<dyn VppApiTransport> = if let Some(afunix_path) = &opts.socket_path {
        Box::new(afunix::Transport::new(&afunix_path))
    } else {
        Box::new(shmem::Transport::new())
    };
    */

    println!("Connect result: {}", t.connect("api-test", None, 256));
    let ping_index: u16 = t.get_msg_index("control_ping_51077d14");
    let cli_inband = t.get_msg_index("cli_inband_f8377302");
    println!("Ping index: {:#x?}", &ping_index);
    let enc = get_encoder();

    let mut v = enc.serialize(&ping_index).unwrap();
    let m = ControlPing {
        client_index: t.get_client_index(),
        context: 0,
    };

    let enc = get_encoder();

    let msg = enc.serialize(&m).unwrap();
    v.extend_from_slice(&msg);
    println!("MSG: {:#x?}", &v);
    t.write(&v);

    /*
    let m = CliInband {
        client_index: t.get_client_index(),
        context: 0,
        cmd: "show version".try_into().unwrap(),
    };
    let enc = get_encoder();
    let mut v = enc.serialize(&cli_inband).unwrap();
    let enc = get_encoder();
    let msg = enc.serialize(&m).unwrap();
    v.extend_from_slice(&msg);
    println!("MSG: {:#x?}", &v);
    t.write(&v);
    */

    let show_threads = t.get_msg_index("show_threads_51077d14");
    let m = ShowThreads {
        client_index: t.get_client_index(),
        context: 0,
    };
    let enc = get_encoder();
    let mut v = enc.serialize(&show_threads).unwrap();
    let enc = get_encoder();
    let msg = enc.serialize(&m).unwrap();
    v.extend_from_slice(&msg);
    println!("MSG: {:#x?}", &v);
    t.write(&v);

    std::thread::sleep(std::time::Duration::from_secs(1));
    let mut buf = [0; 2048];
    println!("Reading");
    // let res = t.read(&mut buf);
    let res = t.read_one_msg_id_and_msg();
    println!("Read1: {:x?}", res);
    let res = t.read_one_msg_id_and_msg();
    println!("Read2: {:x?}", &res);
    if let Ok((msg_id, data)) = res {
        println!("Original data len: {}", data.len());
        println!("from  a bin: {:x?}", &data);
        let r: ShowThreadsReply = get_encoder().deserialize(&data).unwrap();
        println!("{:?}", &r);
        let data = serde_json::to_string(&r).unwrap();
        println!("JSON: {}", data);
        let enc = get_encoder();
        let mut v = enc.serialize(&r).unwrap();
        println!("back to bin: {:x?}", &v);
        println!("New data len: {}", v.len());
    }
    let res = t.read_one_msg_id_and_msg();
    println!("Read3: {:x?}", res);
    // t.control_ping();
    //
    // bench(&mut t);

    std::thread::sleep(std::time::Duration::from_secs(1));
    t.disconnect();

    // std::thread::sleep(std::time::Duration::from_secs(1));
}
