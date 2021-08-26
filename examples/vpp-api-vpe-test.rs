use bincode::Options;
use clap::Clap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/* 
Things I did here 
1. Test Show VPE Time 
2. Associated Methods to message structs to fetch message id 
3. The message ID as of now is hardcoded but the same can be done while generating the api bindings 
4. Use type instead of Unit structs for better code readability and consistent with aliases 
5. Error handling done because I didn't want my terminal to have lot of warnings 
*/

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
pub struct ShowVPESystemTime{
    pub client_index: u32, 
    pub context: u32, 
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowVPESystemTimeReply{
    pub context: u32,
    pub retval: i32, 
    pub vpe_system_time: Timestamp
}
type Timestamp = f64;

impl ShowVPESystemTime{
    fn message_name() -> &'static str {
        "show_vpe_system_time"
    }
    fn message_crc() -> &'static str {
       "51077d14"
    }
    fn message_id() -> String{
        format!("{}_{}", ShowVPESystemTime::message_name(), ShowVPESystemTime::message_crc())
    }
}
impl ShowVPESystemTimeReply{
    fn message_name() -> &'static str {
        "show_vpe_system_time_reply"
    }
    fn message_crc() -> &'static str {
       "7ffd8193"
    }
    fn message_id() -> String{
        format!("{}_{}", ShowVPESystemTimeReply::message_name(), ShowVPESystemTimeReply::message_crc())
    }
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
    match t.write(&v){
        Result::Err(_) => {println!("Failed to write to VPP")},
        Result::Ok(_) => {}
    }

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
    // test_func();
    // let mut t = shmem::Transport::new();
    // let mut t = afunix::Transport::new("/tmp/api.sock");
    let mut t: Box<dyn VppApiTransport> = if let Some(afunix_path) = &opts.socket_path {
        Box::new(afunix::Transport::new(&afunix_path))
    } else {
        Box::new(shmem::Transport::new())
    };

    println!("Connect result: {:?}", t.connect("api-test", None, 256));
    match t.set_nonblocking(opts.nonblocking){
        Result::Err(_err) => println!("Setting non blocking failed"),
        Result::Ok(_ok) => {}
    }

    let get_vpe_time: ShowVPESystemTimeReply = send_recv_msg(
        &ShowVPESystemTime::message_id(),
        &ShowVPESystemTime{
            client_index: t.get_client_index(),
            context: 0
        },
        &mut *t,
        &ShowVPESystemTimeReply::message_id()
    );
    println!("Show VPE Time Reply: {:#?}", &get_vpe_time);
    // t.control_ping();
    //
    // bench(&mut *t);

    std::thread::sleep(std::time::Duration::from_secs(1));
    t.disconnect();

    // std::thread::sleep(std::time::Duration::from_secs(1));
}
