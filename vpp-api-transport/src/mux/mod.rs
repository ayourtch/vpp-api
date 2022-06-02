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

/*
 * The envelope encapsulating the messages traveling "upstream" - from fanout transport and Beacon
 * to Mux thread.
 */
#[derive(Debug)]
enum UpstreamMessage {
    CreateFanout(DownstreamSender),
    Connect((String, Option<String>, i32)),
    DataReady,
    Msg((usize, Vec<u8>)),
}

/*
 * The envelope encapsulating the messages traveling "downstream" - from Mux to a fanout transport.
 */
#[derive(Debug)]
enum DownstreamMessage {
    CreateFanoutResult(Result<UpstreamSender>),
    ConnectResult(Result<()>),
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

fn new<T: VppApiTransport>(real_transport: T) -> Muxer<T> {
    // (Beacon, Fanout) -> Mux mpsc queue
    let (upstream_tx, upstream_rx) = channel::<UpstreamMessage>();
    Muxer {
        real_transport,
        real_transport_connected: None,
        fanout: vec![],
        free_sender_index: 0,
        upstream_tx,
        upstream_rx,
    }
}

impl<T: VppApiTransport> Drop for Muxer<T> {
    fn drop(&mut self) {
        // self.real_transport.disconnect();
    }
}

impl<T: VppApiTransport> Muxer<T> {
    pub fn new_transport(&mut self) -> Transport<T> {
        Transport {
            index: 0,
            phantom: PhantomData,
        }
    }
}

pub struct Transport<T>
where
    T: VppApiTransport,
{
    index: usize,
    phantom: PhantomData<T>,
}

impl<T: VppApiTransport> Transport<T> {}

impl<T: VppApiTransport> Drop for Transport<T> {
    fn drop(&mut self) {}
}

impl<T: VppApiTransport> std::io::Read for Transport<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // FIXME
        Ok(0)
    }
}
impl<T: VppApiTransport> std::io::Write for Transport<T> {
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

impl<T: VppApiTransport> VppApiTransport for Transport<T> {
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
