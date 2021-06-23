
use serde::ser::{ SerializeSeq};
use serde::{Deserialize, Serialize, Serializer};
use serde::de::{ Deserializer, SeqAccess, Visitor};
use crate::*;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VppJsApiMessageInfo {
    pub crc: String,
}

#[derive(Debug, Clone)]
pub struct VppJsApiMessage {
    pub name: String,
    pub fields: Vec<VppJsApiMessageFieldDef>,
    pub info: VppJsApiMessageInfo,
}

impl Serialize for VppJsApiMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(1 + self.fields.len() + 1))?;
        seq.serialize_element(&self.name)?;
        for e in &self.fields {
            seq.serialize_element(e)?;
        }
        seq.serialize_element(&self.info)?;
        seq.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VppJsApiMessageHelper {
    Field(VppJsApiMessageFieldDef),
    Info(VppJsApiMessageInfo),
    Name(String),
}

pub struct VppJsApiMessageVisitor;

impl<'de> Visitor<'de> for VppJsApiMessageVisitor {
    type Value = VppJsApiMessage;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct VppJsApiMessage")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<VppJsApiMessage, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let name: String = if let Some(VppJsApiMessageHelper::Name(s)) = seq.next_element()? {
            s
        } else {
            panic!("Error");
        };
        log::debug!("API message: {}", &name);
        let mut fields: Vec<VppJsApiMessageFieldDef> = vec![];
        let mut maybe_info: Option<VppJsApiMessageInfo> = None;
        loop {
            let nxt = seq.next_element();
            log::debug!("Next: {:#?}", &nxt);
            match nxt? {
                Some(VppJsApiMessageHelper::Field(f)) => fields.push(f),
                Some(VppJsApiMessageHelper::Info(i)) => {
                    if maybe_info.is_some() {
                        panic!("Info is already set!");
                    }
                    maybe_info = Some(i);
                    break;
                }
                x => panic!("Unexpected element {:?}", x),
            }
        }
        let info = maybe_info.unwrap();
        Ok(VppJsApiMessage { name, fields, info })
    }
}

impl<'de> Deserialize<'de> for VppJsApiMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(VppJsApiMessageVisitor)
    }
}