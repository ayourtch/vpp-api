use clap::Clap;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize, Serializer};
use std::string::ToString;
extern crate strum;
#[macro_use]
extern crate strum_macros;
use env_logger;
use linked_hash_map::LinkedHashMap;

#[derive(Clap, Debug, Serialize, Deserialize, EnumString, Display)]
enum OptParseType {
    File,
    Tree,
    ApiType,
    ApiMessage,
}

/// Ingest the VPP API JSON definition file and output the Rust code
#[clap(version = "0.1", author = "Andrew Yourtchenko <ayourtch@gmail.com>")]
#[derive(Clap, Debug, Serialize, Deserialize)]
struct Opts {
    /// Input file name
    #[clap(short, long)]
    in_file: String,

    /// output file name
    #[clap(short, long, default_value = "dummy.rs")]
    out_file: String,

    /// parse type for the operation: Tree, File, ApiMessage or ApiType
    #[clap(short, long, default_value = "File")]
    parse_type: OptParseType,

    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

#[derive(Debug)]
struct VppApiType {
    type_name: String,
    fields: Vec<VppApiMessageFieldDef>,
}

impl Serialize for VppApiType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(1 + self.fields.len()))?;
        seq.serialize_element(&self.type_name)?;
        for e in &self.fields {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt;

struct VppApiTypeVisitor;

impl<'de> Visitor<'de> for VppApiTypeVisitor {
    type Value = VppApiType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct VppApiType")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<VppApiType, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let type_name: String = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let mut fields: Vec<VppApiMessageFieldDef> = vec![];
        loop {
            let nxt = seq.next_element();
            log::debug!("Next: {:#?}", &nxt);
            if let Ok(Some(v)) = nxt {
                fields.push(v);
            } else {
                break;
            }
        }
        Ok(VppApiType { type_name, fields })
    }
}

impl<'de> Deserialize<'de> for VppApiType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(VppApiTypeVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum VppApiDefaultValue {
    Str(String),
    Bool(bool),
    I64(i64),
    F64(f64),
}

#[derive(Debug, Serialize, Deserialize)]
struct VppApiFieldOptions {
    #[serde(default)]
    default: Option<VppApiDefaultValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum VppApiFieldSize {
    Fixed(usize),
    Variable(Option<String>),
}

#[derive(Debug)]
struct VppApiMessageFieldDef {
    ctype: String,
    name: String,
    maybe_size: Option<VppApiFieldSize>,
    maybe_options: Option<VppApiFieldOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum VppApiMessageFieldHelper {
    Str(String),
    Usize(usize),
    Map(VppApiFieldOptions),
}

impl Serialize for VppApiMessageFieldDef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use crate::VppApiFieldSize::*;

        let mut len = 2;
        if self.maybe_options.is_some() {
            len = len + 1
        }
        len = len
            + match &self.maybe_size {
                None => 0,
                Some(Fixed(n)) => 1,
                Some(Variable(None)) => 1,
                Some(Variable(Some(x))) => 2,
            };
        let mut seq = serializer.serialize_seq(Some(len))?;
        seq.serialize_element(&self.ctype)?;
        seq.serialize_element(&self.name)?;
        match &self.maybe_size {
            None => { /* do nothing */ }
            Some(Fixed(n)) => {
                seq.serialize_element(&n);
            }
            Some(Variable(None)) => {
                seq.serialize_element(&0u32);
            }
            Some(Variable(Some(x))) => {
                seq.serialize_element(&0u32);
                seq.serialize_element(&x);
            }
        }

        if let Some(o) = &self.maybe_options {
            seq.serialize_element(o);
        }
        seq.end()
    }
}

struct VppApiMessageFieldDefVisitor;

impl<'de> Visitor<'de> for VppApiMessageFieldDefVisitor {
    type Value = VppApiMessageFieldDef;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct VppApiMessageField")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<VppApiMessageFieldDef, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let ctype: String = if let Some(VppApiMessageFieldHelper::Str(s)) = seq.next_element()? {
            s
        } else {
            panic!("Error");
        };
        let name: String = if let Some(VppApiMessageFieldHelper::Str(s)) = seq.next_element()? {
            s
        } else {
            panic!("Error 2");
        };

        let mut maybe_sz: Option<usize> = None;
        let mut maybe_sz_name: Option<String> = None;
        let mut maybe_options: Option<VppApiFieldOptions> = None;

        loop {
            let nxt = seq.next_element();
            match nxt? {
                Some(VppApiMessageFieldHelper::Map(m)) => {
                    maybe_options = Some(m);
                    break;
                }
                Some(VppApiMessageFieldHelper::Str(o)) => {
                    maybe_sz_name = Some(o);
                }
                Some(VppApiMessageFieldHelper::Usize(o)) => {
                    maybe_sz = Some(o);
                }
                None => break,
            }
        }
        let maybe_size = match (maybe_sz, maybe_sz_name) {
            (None, None) => None,
            (Some(0), None) => Some(VppApiFieldSize::Variable(None)),
            (Some(0), Some(s)) => Some(VppApiFieldSize::Variable(Some(s))),
            (Some(x), None) => Some(VppApiFieldSize::Fixed(x)),
            (None, Some(s)) => panic!("Unexpected dependent field {} with no length", s),
            (Some(x), Some(s)) => panic!("Unexpected dependent field {} with length {}", s, x),
        };
        let ret = VppApiMessageFieldDef {
            ctype,
            name,
            maybe_size,
            maybe_options,
        };
        Ok(ret)
    }
}

impl<'de> Deserialize<'de> for VppApiMessageFieldDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(VppApiMessageFieldDefVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct VppApiMessageInfo {
    crc: String,
}

#[derive(Debug)]
struct VppApiMessage {
    name: String,
    fields: Vec<VppApiMessageFieldDef>,
    info: VppApiMessageInfo,
}

impl Serialize for VppApiMessage {
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum VppApiMessageHelper {
    Field(VppApiMessageFieldDef),
    Info(VppApiMessageInfo),
    Name(String),
}

struct VppApiMessageVisitor;

impl<'de> Visitor<'de> for VppApiMessageVisitor {
    type Value = VppApiMessage;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct VppApiMessage")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<VppApiMessage, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let name: String = if let Some(VppApiMessageHelper::Name(s)) = seq.next_element()? {
            s
        } else {
            panic!("Error");
        };
        log::debug!("API message: {}", &name);
        let mut fields: Vec<VppApiMessageFieldDef> = vec![];
        let mut maybe_info: Option<VppApiMessageInfo> = None;
        loop {
            let nxt = seq.next_element();
            log::debug!("Next: {:#?}", &nxt);
            match nxt? {
                Some(VppApiMessageHelper::Field(f)) => fields.push(f),
                Some(VppApiMessageHelper::Info(i)) => {
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
        Ok(VppApiMessage { name, fields, info })
    }
}

impl<'de> Deserialize<'de> for VppApiMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(VppApiMessageVisitor)
    }
}

#[derive(Debug, Deserialize)]
struct VppApiAlias {
    #[serde(rename = "type")]
    ctype: String,
    length: Option<usize>,
}

impl Serialize for VppApiAlias {
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

#[derive(Debug, Deserialize)]
struct VppApiService {
    reply: String,
    stream: Option<bool>,
}

impl Serialize for VppApiService {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 1;
        if self.stream.is_some() {
            len = len + 1;
        }
        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("reply", &self.reply)?;
        if let Some(s) = &self.stream {
            map.serialize_entry("stream", s);
        }
        map.end()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct VppApiOptions {
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VppApiEnumInfo {
    enumtype: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VppApiEnumValueDef {
    name: String,
    value: i64,
}

impl Serialize for VppApiEnumValueDef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&self.value)?;
        seq.end()
    }
}

#[derive(Debug)]
struct VppApiEnum {
    name: String,
    values: Vec<VppApiEnumValueDef>,
    info: VppApiEnumInfo,
}

impl Serialize for VppApiEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(1 + self.values.len() + 1))?;
        seq.serialize_element(&self.name)?;
        for e in &self.values {
            seq.serialize_element(e)?;
        }
        seq.serialize_element(&self.info)?;
        seq.end()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum VppApiEnumHelper {
    Str(String),
    Val(VppApiEnumValueDef),
    Map(VppApiEnumInfo),
}

struct VppApiEnumVisitor;

impl<'de> Visitor<'de> for VppApiEnumVisitor {
    type Value = VppApiEnum;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct VppApiEnum")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<VppApiEnum, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let name: String = if let Some(VppApiEnumHelper::Str(s)) = seq.next_element()? {
            s
        } else {
            panic!("Error");
        };
        log::debug!("API message: {}", &name);
        let mut values: Vec<VppApiEnumValueDef> = vec![];
        let mut maybe_info: Option<VppApiEnumInfo> = None;
        loop {
            let nxt = seq.next_element();
            log::debug!("Next: {:#?}", &nxt);
            match nxt? {
                Some(VppApiEnumHelper::Val(f)) => values.push(f),
                Some(VppApiEnumHelper::Map(i)) => {
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
        Ok(VppApiEnum { name, values, info })
    }
}

impl<'de> Deserialize<'de> for VppApiEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(VppApiEnumVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct VppApiFile {
    types: Vec<VppApiType>,
    messages: Vec<VppApiMessage>,
    unions: Vec<VppApiType>,
    enums: Vec<VppApiEnum>,
    #[serde(default)]
    enumflags: Vec<VppApiEnum>,
    services: LinkedHashMap<String, VppApiService>,
    options: VppApiOptions,
    aliases: LinkedHashMap<String, VppApiAlias>,
    vl_api_version: String,
    imports: Vec<String>,
}

fn parse_api_tree(opts: &Opts, root: &str, map: &mut LinkedHashMap<String, VppApiFile>) {
    use std::fs;
    if opts.verbose > 2 {
        println!("parse tree: {:?}", root);
    }
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if opts.verbose > 2 {
            println!("Entry: {:?}", &entry);
        }

        let metadata = fs::metadata(&path).unwrap();
        if metadata.is_file() {
            let res = std::fs::read_to_string(&path);
            if let Ok(data) = res {
                let desc = serde_json::from_str::<VppApiFile>(&data);
                if let Ok(d) = desc {
                    map.insert(path.to_str().unwrap().to_string(), d);
                } else {
                    eprintln!("Error loading {:?}: {:?}", &path, &desc);
                }
            } else {
                eprintln!("Error reading {:?}: {:?}", &path, &res);
            }
        }
        if metadata.is_dir() && entry.file_name() != "." && entry.file_name() != ".." {
            parse_api_tree(opts, &path.to_str().unwrap(), map);
        }
    }
}

fn main() {
    env_logger::init();
    let opts: Opts = Opts::parse();
    log::info!("Starting file {}", &opts.in_file);

    if let Ok(data) = std::fs::read_to_string(&opts.in_file) {
        match opts.parse_type {
            OptParseType::Tree => {
                panic!("Can't parse a tree out of file!");
            }
            OptParseType::File => {
                let desc: VppApiFile = serde_json::from_str(&data).unwrap();
                eprintln!(
                    "File: {} version: {} services: {} types: {} messages: {} aliases: {} imports: {} enums: {} unions: {}",
                    &opts.in_file,
                    &desc.vl_api_version,
                    desc.services.len(),
                    desc.types.len(),
                    desc.messages.len(),
                    desc.aliases.len(),
                    desc.imports.len(),
                    desc.enums.len(),
                    desc.unions.len()
                );
                if opts.verbose > 1 {
                    println!("Dump File: {:#?}", &desc);
                }
                let data = serde_json::to_string_pretty(&desc).unwrap();
                println!("{}", &data);
            }
            OptParseType::ApiType => {
                let desc: VppApiType = serde_json::from_str(&data).unwrap();
                println!("Dump Type: {:#?}", &desc);
            }
            OptParseType::ApiMessage => {
                let desc: VppApiMessage = serde_json::from_str(&data).unwrap();
                println!("Dump: {:#?}", &desc);
            }
        }
    } else {
        match opts.parse_type {
            OptParseType::Tree => {
                // it was a directory tree, descend downwards...
                let mut api_files: LinkedHashMap<String, VppApiFile> = LinkedHashMap::new();
                parse_api_tree(&opts, &opts.in_file, &mut api_files);
                println!("Loaded {} API definition files", api_files.len());
            }
            e => {
                panic!("inappropriate parse type {:?} for inexistent file", e);
            }
        }
    }
}
