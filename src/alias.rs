use clap::Clap;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize, Serializer};
use std::string::ToString;
extern crate strum;
#[macro_use]
use env_logger;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt;
use crate::*;

#[derive(Debug, Deserialize)]
pub struct VppJsApiAlias {
    #[serde(rename = "type")]
    pub ctype: String,
    pub length: Option<usize>,
}

impl Serialize for VppJsApiAlias {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 1;
        if self.length.is_some() {
            len = len + 1;
        }
        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("type", &self.ctype)?;
        if let Some(s) = &self.length {
            map.serialize_entry("length", s);
        }
        map.end()
    }
}