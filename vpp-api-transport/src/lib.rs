#[macro_use]
extern crate lazy_static;

#[macro_use]
mod macros;
pub mod afunix;
pub mod shmem;
// Interactions. May be evicted later on...
pub mod error;
pub mod reqrecv;
use crate::error::Error;
use crate::error::Result;
use bincode;
use bincode::Options;
use lazy_static::__Deref;
use log::debug;
use log::warn;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::io::{Read, Write};
use std::ops::DerefMut;

#[derive(Serialize, Deserialize, Debug)]
struct SockMsgHeader {
    _q: u64,
    msglen: u32,
    gc_mark: u32,
}

fn get_encoder() -> impl bincode::config::Options {
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawControlPing {
    _vl_msg_id: u16,
    client_index: u32,
    context: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawControlPingReply {
    context: u32,
    retval: i32,
    client_index: u32,
    vpe_pid: u32,
}

/* This is a pretty hacky way to convince bincode, but oh well... */
#[derive(Debug, Clone)]
enum VarLen32 {
    VarLenData(Vec<u8>),
}

use serde::ser::{SerializeTuple, Serializer};

impl Serialize for VarLen32 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = match self {
            VarLen32::VarLenData(v) => v,
        };

        let len = data.len();
        let mut seq = serializer.serialize_tuple(len + 4)?;
        let b0: u8 = (len >> 24) as u8;
        let b1: u8 = ((len >> 16) & 0xff) as u8;
        let b2: u8 = ((len >> 8) & 0xff) as u8;
        let b3: u8 = (len & 0xff) as u8;
        seq.serialize_element(&b0)?;
        seq.serialize_element(&b1)?;
        seq.serialize_element(&b2)?;
        seq.serialize_element(&b3)?;
        for b in data {
            seq.serialize_element(b)?;
        }
        seq.end()
    }
}

use serde::de::{Deserializer, SeqAccess, Visitor};
use std::fmt;

impl<'de> Deserialize<'de> for VarLen32 {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VarLen32Visitor;
        impl<'de> Visitor<'de> for VarLen32Visitor {
            type Value = VarLen32;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("VarLen32")
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut res: Vec<u8> = vec![];

                let length: u32 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

                for _i in 0..length {
                    res.push(
                        seq.next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?,
                    );
                }

                return Ok(VarLen32::VarLenData(res));
            }
        }

        return Ok(deserializer.deserialize_tuple(1 << 16, VarLen32Visitor)?);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawCliInband {
    _vl_msg_id: u16,
    client_index: u32,
    context: u32,
    cmd: VarLen32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawCliInbandReply {
    context: u32,
    retval: i32,
    reply: VarLen32,
}

pub trait VppApiTransport: Read + Write {
    fn connect(&mut self, name: &str, chroot_prefix: Option<&str>, rx_qlen: i32) -> Result<()>;
    fn disconnect(&mut self);
    fn set_nonblocking(&mut self, nonblocking: bool) -> Result<()>;

    fn get_msg_index(&mut self, name: &str) -> Option<u16>;
    fn get_table_max_index(&mut self) -> u16;
    fn get_client_index(&self) -> u32;

    fn get_next_context(&mut self) -> u32 {
        // FIXME: use atomic autoincrementing
        42
    }

    fn control_ping(&mut self) -> std::io::Result<u32> {
        let control_ping_id = self.get_msg_index("control_ping_51077d14").unwrap();
        let context = self.get_next_context();
        let msg = RawControlPing {
            _vl_msg_id: control_ping_id,
            client_index: self.get_client_index(),
            context,
        };
        let data = get_encoder().serialize(&msg).unwrap();
        self.write(&data)?;
        Ok(context)
    }

    fn skip_to_control_ping_reply(&mut self, _context: u32) -> Result<()> {
        let control_ping_reply_id = self.get_msg_index("control_ping_reply_f6b0b8ca").unwrap();
        loop {
            match self.read_one_msg_id_and_msg() {
                Err(e) => return Err(e),
                Ok((msg_id, _data)) => {
                    if msg_id == control_ping_reply_id {
                        // FIXME: deserialize and match the context
                        return Ok(());
                    }
                }
            }
        }
    }

    fn run_cli_inband(&mut self, cmd: &str) -> Result<String> {
        let cli_inband_id = self.get_msg_index("cli_inband_f8377302").unwrap();
        let cli_inband_reply_id = self.get_msg_index("cli_inband_reply_05879051").unwrap();

        let context = self.get_next_context();
        let msg = RawCliInband {
            _vl_msg_id: cli_inband_id,
            client_index: self.get_client_index(),
            context,
            cmd: VarLen32::VarLenData(cmd.as_bytes().to_vec()),
        };
        let data = get_encoder().serialize(&msg).unwrap();
        // println!("Sending data: {:?}", &data);
        self.write(&data)?;

        loop {
            match self.read_one_msg_id_and_msg() {
                Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => {
                    return Err(e);
                }
                Ok((msg_id, data)) => {
                    if msg_id == cli_inband_reply_id {
                        // println!("Message: {:?}", &data);
                        let r: RawCliInbandReply = get_encoder().deserialize(&data).unwrap();
                        let v = match r.reply {
                            VarLen32::VarLenData(d) => d,
                        };
                        let s = String::from_utf8_lossy(&v);
                        // println!("Command reply: {}", &s);
                        return Ok(s.to_string());
                    }
                }
            }
        }
    }

    fn dump(&self);

    fn read_one_msg_into(&mut self, data: &mut Vec<u8>) -> Result<()> {
        let mut header_buf = [0; 16];

        if let Err(e) = self.read_exact(&mut header_buf) {
            warn!("read invalid header: {:?} err: {:?}", header_buf, e);
            return Err(Error::InvalidHeader);
        }

        let hdr: SockMsgHeader = get_encoder().deserialize(&header_buf[..])?;
        debug!("Got header: {:?}", hdr);

        match hdr.msglen.try_into() {
            Ok(msglen) => {
                if msglen == 0 {
                    return Err(Error::InvalidMessage);
                }
                data.resize(msglen, 0);
                if let Err(e) = self.read_exact(data) {
                    warn!("expected {} byte message, got error: {:?}", msglen, e);
                    return Err(Error::InvalidMessage);
                }
                Ok(())
            }
            Err(e) => Err(Error::Error(format!(
                "msg length {} couldn't be converted to usize: {}",
                hdr.msglen, e
            ))),
        }
    }

    fn read_one_msg(&mut self) -> Result<Vec<u8>> {
        let mut out: Vec<u8> = vec![];
        self.read_one_msg_into(&mut out)?;
        Ok(out)
    }

    fn read_one_msg_id_and_msg(&mut self) -> Result<(u16, Vec<u8>)> {
        let ret = self.read_one_msg()?;
        if ret.len() < 3 {
            return Err(format!("short read message len: {}  {:x?}", ret.len(), ret).into());
        }
        let msg_id: u16 = ((ret[0] as u16) << 8) + (ret[1] as u16);
        Ok((msg_id, ret[2..].to_vec()))
    }
}

impl<T> VppApiTransport for Box<T>
where
    T: VppApiTransport,
{
    fn connect(&mut self, name: &str, chroot_prefix: Option<&str>, rx_qlen: i32) -> Result<()> {
        self.deref_mut().connect(name, chroot_prefix, rx_qlen)
    }

    fn disconnect(&mut self) {
        self.deref_mut().disconnect()
    }

    fn set_nonblocking(&mut self, nonblocking: bool) -> Result<()> {
        self.deref_mut().set_nonblocking(nonblocking)
    }

    fn get_msg_index(&mut self, name: &str) -> Option<u16> {
        self.deref_mut().get_msg_index(name)
    }

    fn get_table_max_index(&mut self) -> u16 {
        self.deref_mut().get_table_max_index()
    }

    fn get_client_index(&self) -> u32 {
        self.deref().get_client_index()
    }

    fn dump(&self) {
        self.deref().dump()
    }
}

#[cfg(test)]
mod tests {
    use crate::afunix;
    use crate::shmem;
    use crate::VppApiTransport;

    #[test]
    fn test_shmem_connect() {
        let mut t1 = shmem::Transport::new();
        let res = t1.connect("test", None, 32);
        assert!(res.is_ok(), "Should be able to connect over shmem");
        t1.disconnect();
        drop(t1);
    }

    #[test]
    fn test_afunix_connect() {
        let mut t1 = afunix::Transport::new("/run/vpp/api.sock");
        let res = t1.connect("test", None, 32);
        assert!(res.is_ok(), "Should be able to connect over afunix");
        let context = t1.control_ping();
        assert!(context.is_ok(), "Should return the context");
        let context = context.unwrap();
        let res = t1.skip_to_control_ping_reply(context);
        assert!(
            res.is_ok(),
            "Should skip up to the matching ping reply and consume it"
        );
        let s = t1.run_cli_inband("show version");
        assert!(s.is_ok(), "should be able to run a CLI");
        let s = s.unwrap();
        assert!(s.starts_with("vpp "));
        t1.disconnect();
        drop(t1);
    }
}
