use generic_array::{ArrayLength, GenericArray};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;
use typenum::{U10, U256, U32, U64};

#[derive(Clone)]
pub struct FixedSizeString<N: ArrayLength<u8>>(GenericArray<u8, N>);

impl<N: ArrayLength<u8>> fmt::Debug for FixedSizeString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = &self.0;
        let val_str = match std::str::from_utf8(v) {
            Ok(s) => format!("{:?}", &s.trim_end_matches("\u{0}")),
            Err(_) => format!("{:?}", &v),
        };
        write!(f, "FixedSizeString[{}]: {}", &N::to_u32(), &val_str)
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

            Ok(FixedSizeString(out))
        }
    }
}

impl<N: ArrayLength<u8>> Serialize for FixedSizeString<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = &self.0;

        let mut len = data.len();
        let mut seq = serializer.serialize_tuple(len)?;
        for b in data {
            seq.serialize_element(b)?;
        }
        seq.end()
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

                return Ok(FixedSizeString(res));
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

#[derive(Clone)]
pub struct VariableSizeString(Vec<u8>);

impl fmt::Debug for VariableSizeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = &self.0;
        let val_str = match std::str::from_utf8(v) {
            Ok(s) => format!("{:?}", &s),
            Err(_) => format!("{:?}", &v),
        };
        write!(f, "VariableSizeString: {}", &val_str)
    }
}

impl TryFrom<&str> for VariableSizeString {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut out: Vec<u8> = vec![];
        for b in value.as_bytes().into_iter() {
            out.push(*b);
        }

        Ok(VariableSizeString(out))
    }
}

impl Serialize for VariableSizeString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = &self.0;

        let len: usize = data.len();
        let len_u32: u32 = len.try_into().unwrap();
        let mut seq = serializer.serialize_tuple(len)?;
        seq.serialize_element(&len_u32)?;
        for b in data {
            seq.serialize_element(&b)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for VariableSizeString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VariableSizeStringVisitor;
        impl<'de> Visitor<'de> for VariableSizeStringVisitor {
            type Value = VariableSizeString;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("VariableSizeString")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut res: Vec<u8> = vec![];
                let length: u32 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

                for i in 0..length {
                    res.push(
                        seq.next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?,
                    );
                }

                return Ok(VariableSizeString(res));
            }
        }

        return Ok(deserializer.deserialize_tuple(1 << 31, VariableSizeStringVisitor)?);
    }
}

#[derive(Clone, Debug)]
pub struct VariableSizeArray<T>(pub Vec<T>);

use serde::ser::{SerializeSeq, SerializeTuple, Serializer};

impl<T: Debug + Serialize> Serialize for VariableSizeArray<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = &self.0;

        let mut len = data.len();
        let mut seq = serializer.serialize_tuple(len)?;
        for b in data {
            seq.serialize_element(&b)?;
        }
        seq.end()
    }
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
                    if nxt.is_err() {
                        break;
                    }
                    let nxt = nxt?;
                    if nxt.is_some() {
                        let nxt: T = nxt.unwrap();
                        res.push(nxt);
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

                return Ok(VariableSizeArray::<T>(res));
            }
        }

        return Ok(deserializer.deserialize_tuple(
            1 << 31,
            VariableSizeArrayVisitor {
                marker: PhantomData,
            },
        )?);
    }
}
