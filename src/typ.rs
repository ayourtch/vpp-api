use generic_array::{ArrayLength, GenericArray};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;
use typenum::U10;

#[derive(Clone, Serialize, Deserialize)]
#[serde(bound = "N: ArrayLength<u8>")]
pub enum FixedSizeString<N: ArrayLength<u8>> {
    FixedSizeString(GenericArray<u8, N>),
}

impl<N: ArrayLength<u8>> fmt::Debug for FixedSizeString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FixedSizeString(v) => {
                let val_str = match std::str::from_utf8(v) {
                    Ok(s) => format!("{:?}", &s),
                    Err(_) => format!("{:?}", &v),
                };
                write!(f, "FixedSizeString[{}]: {}", &N::to_u32(), &val_str)
            }
        }
    }
}

impl<N: ArrayLength<u8>> TryFrom<&str> for FixedSizeString<N> {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut out: GenericArray<u8, N> = Default::default();
        let max_len = out.len() - 1;
        if value.len() > max_len {
            Err(format!(
                "The source length of {:?} is {} > max {}",
                value,
                value.len(),
                max_len
            ))
        } else {
            for (i, b) in value.as_bytes().into_iter().enumerate() {
                out[i] = *b;
            }

            Ok(Self::FixedSizeString(out))
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum VariableSizeString {
    VariableSizeString(Vec<u8>),
}

impl fmt::Debug for VariableSizeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VariableSizeString(v) => {
                let val_str = match std::str::from_utf8(v) {
                    Ok(s) => format!("{:?}", &s),
                    Err(_) => format!("{:?}", &v),
                };
                write!(f, "VariableSizeString: {}", &val_str)
            }
        }
    }
}

impl TryFrom<&str> for VariableSizeString {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut out: Vec<u8> = vec![];
        for b in value.as_bytes().into_iter() {
            out.push(*b);
        }

        Ok(Self::VariableSizeString(out))
    }
}

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

pub fn test_func() {
    let t = CliInband {
        client_index: 0xaaaabbbb,
        context: 0xccccdddd,
        cmd: "testng123".try_into().unwrap(),
    };
    println!("t: {:#x?}", &t);
}
