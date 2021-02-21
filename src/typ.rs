use generic_array::{ArrayLength, GenericArray};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;
use typenum::{U10, U64};

#[derive(Clone, Serialize)]
#[serde(bound = "N: ArrayLength<u8>")]
pub enum FixedSizeString<N: ArrayLength<u8>> {
    FixedSizeString(GenericArray<u8, N>),
}

impl<N: ArrayLength<u8>> fmt::Debug for FixedSizeString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FixedSizeString(v) => {
                let val_str = match std::str::from_utf8(v) {
                    Ok(s) => format!("{:?}", &s.trim_end_matches("\u{0}")),
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

impl<'de, N: ArrayLength<u8>> Deserialize<'de> for FixedSizeString<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FixedSizeStringVisitor<N> {
            marker: PhantomData<N>,
        };
        impl<'de, N> Visitor<'de> for FixedSizeStringVisitor<N>
        where
            N: ArrayLength<u8>,
        {
            type Value = FixedSizeString<N>;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("FixedSizeString")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut res: GenericArray<u8, N> = Default::default();
                let length = N::to_u32() as usize;

                for i in 0..length {
                    let b = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                    res[i] = b;
                }

                return Ok(FixedSizeString::FixedSizeString(res));
            }
        }

        return Ok(deserializer.deserialize_tuple(
            N::to_u32() as usize,
            FixedSizeStringVisitor {
                marker: PhantomData,
            },
        )?);
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

#[derive(Clone, Debug, Serialize)]
pub enum VariableSizeArray<T> {
    VariableSizeData(Vec<T>),
}

use core::marker::PhantomData;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt::Debug;
// use std::fmt;

impl<'de, T: Deserialize<'de> + Debug> Deserialize<'de> for VariableSizeArray<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VariableSizeArrayVisitor<T> {
            marker: PhantomData<T>,
        };
        impl<'de, T> Visitor<'de> for VariableSizeArrayVisitor<T>
        where
            T: Deserialize<'de> + Debug,
        {
            type Value = VariableSizeArray<T>;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("VariableSizeArray")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut res: Vec<T> = vec![];
                /*
                                let length: u32 = seq
                                    .next_element()?
                                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                */

                loop {
                    let nxt = seq.next_element();
                    if nxt.is_ok() {
                        let nxt = nxt.unwrap();
                        if nxt.is_none() {
                            break;
                        }
                        res.push(nxt.unwrap());
                    }
                }
                /*
                for i in 0..length {
                    res.push(
                        seq.next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?,
                    );
                }
                */

                return Ok(VariableSizeArray::VariableSizeData(res));
            }
        }

        return Ok(deserializer.deserialize_tuple(
            1,
            VariableSizeArrayVisitor {
                marker: PhantomData,
            },
        )?);
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
    thread_data: VariableSizeArray<ThreadData>,
}

pub fn test_func() {
    let t = CliInband {
        client_index: 0xaaaabbbb,
        context: 0xccccdddd,
        cmd: "testng123".try_into().unwrap(),
    };
    println!("t: {:#x?}", &t);
}
