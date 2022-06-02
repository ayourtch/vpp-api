use bincode;
use bincode::Options;
use serde::{Deserialize, Serialize};
use std::os::unix::net::UnixStream;

use crate::error::Result;
use crate::VppApiBeaconing;
use crate::VppApiTransport;

use std::collections::HashMap;

use crate::get_encoder;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use std::marker::PhantomData;
use std::sync::mpsc::channel;
use std::thread;

#[derive(Serialize, Deserialize, Debug)]
struct SockMsgHeader {
    _q: u64,
    msglen: u32,
    gc_mark: u32,
}

enum MuxMessage {
    DoClose,
    DoOpen,
}

#[derive(Debug, Default)]
pub struct Muxer<T>
where
    T: VppApiTransport,
{
    real_transport: T,
    real_transport_connected: bool,
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
