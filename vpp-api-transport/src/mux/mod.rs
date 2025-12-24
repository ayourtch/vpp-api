use bincode;
use bincode::Options;
use serde::{Deserialize, Serialize};
use std::os::unix::net::UnixStream;

use crate::error::Result;
use crate::VppApiBeaconing;
use crate::VppApiTransport;

use std::collections::HashMap;

use crate::get_encoder;
use socketpair::SocketpairStream;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use socketpair::socketpair_stream;
use std::marker::PhantomData;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/*
 * Made for the sync model of operation, the Mux transport spawns two threads:
 *
 * 1) the Mux thread: it owns the underlying real transport.
 *    It listens on a single MPSC receiver point, with the senders
 *    being the fanout transports, and also the Beacon thread.
 *
 *    Upon getting a message from Fanout transports, it records the (index, context) mapping,
 *    creates a translation, and sends the contents to the underlying real transport.
 *
 *    Upon getting a message from the Beacon sender, it performs the read
 *    of the messages from the real transport, uses the context to get the (index, context)
 *    mapping, translates the payload and dispatches them to the corresponding fanout transport.
 *
 *    In order to do that, underlying real transport is being set to non-blocking mode,
 *    such that all of the pending messages can be drained out.
 *
 * 2) the Beacon thread: it polls the beacon socket and sends the DataReady
 *    message to the Mux thread when there is data to be read.
 *
 */

#[derive(Debug)]
struct GlobalState {
    muxes: HashMap<String, Transport>,
}

lazy_static! {
    static ref GLOBAL: Arc<Mutex<GlobalState>> = {
        let gs = GlobalState {
            muxes: HashMap::new(),
        };

        Arc::new(Mutex::new(gs))
    };
}

/*
 * The envelope encapsulating the messages traveling "upstream" - from fanout transport and Beacon
 * to Mux thread.
 */
#[derive(Debug)]
enum UpstreamMessage {
    CreateDownstream(DownstreamSender),
    Connect((String, Option<String>, i32)),
    DataReady,
    Msg((usize, Vec<u8>)),
}

/*
 * The envelope encapsulating the messages traveling "downstream" - from Mux to a fanout transport.
 */
#[derive(Debug)]
enum DownstreamMessage {
    CreateDownstreamReply(Result<UpstreamSender>),
    ConnectReply(Result<()>),
    UpstreamSender(Sender<UpstreamMessage>),
    Msg(Vec<u8>),
}

#[derive(Debug)]
struct UpstreamSender {
    sender: Sender<UpstreamMessage>,
    index: usize,
}

#[derive(Debug)]
struct DownstreamSender {
    activator: SocketpairStream,
    sender: Sender<DownstreamMessage>,
}

#[derive(Debug)]
struct DownstreamReceiver {
    beacon: SocketpairStream,
    receiver: Receiver<DownstreamMessage>,
}

#[derive(Debug)]
pub struct TransportFactory {
    mux_sender: UpstreamSender,
}

fn new_downstream_pair() -> Result<(DownstreamSender, DownstreamReceiver)> {
    let (sender, receiver) = channel::<DownstreamMessage>();
    let (activator, beacon) = socketpair_stream()?;
    let tx = DownstreamSender { activator, sender };
    let rx = DownstreamReceiver { beacon, receiver };
    Ok((tx, rx))
}

#[derive(Debug)]
pub struct Muxer<T>
where
    T: VppApiTransport,
{
    real_transport: T,
    real_transport_connected: Option<(String, Option<String>, i32)>,
    fanout: Vec<Option<DownstreamSender>>,
    free_sender_index: usize,
    upstream_tx: Sender<UpstreamMessage>, // template to clone from
    upstream_rx: Receiver<UpstreamMessage>, // receiving end for Mux thread
}

#[derive(Debug)]
pub struct Transport {
    upstream: UpstreamSender,
    downstream: DownstreamReceiver,
}

impl<T: VppApiTransport> Muxer<T> {
    fn new_upstream_sender(&mut self, d: DownstreamSender) -> UpstreamSender {
        let sender = self.upstream_tx.clone();
        let index = self.free_sender_index;
        let utx = UpstreamSender { sender, index };
        if self.free_sender_index >= self.fanout.len() {
            self.fanout.push(Some(d));
            self.free_sender_index = self.free_sender_index + 1;
        } else {
            assert!(self.fanout[self.free_sender_index].is_none());
            self.fanout[self.free_sender_index] = Some(d);
            while self.free_sender_index < self.fanout.len()
                && self.fanout[self.free_sender_index].is_some()
            {
                self.free_sender_index = self.free_sender_index + 1;
            }
        }
        utx
    }
}

pub fn new<T: 'static + VppApiTransport>(key: &str, create_underlay: fn() -> T) -> Transport {
    let mut gs = GLOBAL.lock().unwrap();
    if !gs.muxes.contains_key(key) {
        let (upstream_tx, upstream_rx) = channel::<UpstreamMessage>();
        let (dtx, drx) = new_downstream_pair().unwrap();
        let real_transport = create_underlay();
        let mut mux = Muxer {
            real_transport,
            real_transport_connected: None,
            fanout: vec![],
            free_sender_index: 0,
            upstream_tx,
            upstream_rx,
        };
        // (Beacon, Fanout) -> Mux mpsc queue
        let utx = mux.new_upstream_sender(dtx);

        // spawn the Mux thread (it will spawn the Beacon thread when connected)
        thread::spawn(move || {
            mux.handle();
        });
        let trans = Transport {
            upstream: utx,
            downstream: drx,
        };
        gs.muxes.insert(key.to_string(), trans);
    }
    gs.muxes.get_mut(key).unwrap().new_transport()
}

impl<T: VppApiTransport> Drop for Muxer<T> {
    fn drop(&mut self) {
        // self.real_transport.disconnect();
    }
}

impl<T: VppApiTransport> Muxer<T> {
    fn handle(&mut self) {
        loop {
            match self.upstream_rx.recv() {
                Ok(msg) => match msg {
                    UpstreamMessage::CreateDownstream(dtx) => {
                        let utx = self.new_upstream_sender(dtx);
                        self.fanout[utx.index]
                            .as_ref()
                            .unwrap()
                            .sender
                            .send(DownstreamMessage::CreateDownstreamReply(Ok(utx)));
                    }
                    x => {
                        eprintln!("Unknown message {:?} received in Mux transport handler", x);
                    }
                },
                Err(e) => {
                    eprintln!("Error {:?} while receiving in Mux transport", e);
                }
            }
        }
    }
}

impl Transport {
    pub fn new_transport(&mut self) -> Transport {
        let (dtx, drx) = new_downstream_pair().unwrap();
        self.upstream
            .sender
            .send(UpstreamMessage::CreateDownstream(dtx));
        let res = drx.receiver.recv().unwrap();
        match res {
            DownstreamMessage::CreateDownstreamReply(utx) => {
                let utx = utx.unwrap();
                Transport {
                    upstream: utx,
                    downstream: drx,
                }
            }
            x => {
                panic!("Unexpected message {:?} in reply to CreateDownstream", x);
            }
        }
    }
}

impl Drop for Transport {
    fn drop(&mut self) {}
}

impl std::io::Read for Transport {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // FIXME
        Ok(0)
    }
}
impl std::io::Write for Transport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

type ArrayOf64U8 = [u8; 64];

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgSockClntCreate {
    _vl_msg_id: u16,
    context: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgSockClntCreateReplyHdr {
    _vl_msg_id: u16,
    client_index: u32,
    context: u32,
    response: i32,
    index: u32,
    count: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgSockClntCreateReplyEntry {
    index: u16,
}

impl VppApiTransport for Transport {
    fn connect(&mut self, name: &str, _chroot_prefix: Option<&str>, _rx_qlen: i32) -> Result<()> {
        use std::io::Write;
        // FIXME

        Ok(())
    }
    fn disconnect(&mut self) {
        // FIXME
    }
    fn set_nonblocking(&mut self, nonblocking: bool) -> Result<()> {
        // FIXME
        Ok(())
    }

    fn get_client_index(&self) -> u32 {
        // FIXME
        0
    }
    fn get_msg_index(&mut self, name: &str) -> Option<u16> {
        // FIXME
        None
    }
    fn get_table_max_index(&mut self) -> u16 {
        0
    }

    fn get_beacon_socket(&self) -> std::io::Result<Box<dyn VppApiBeaconing>> {
        Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "FIXME - not implemented").into())
    }

    fn dump(&self) {}
}
