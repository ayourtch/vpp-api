use clap::Clap;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize, Serializer};
use std::string::ToString;
extern crate strum;
#[macro_use]
extern crate strum_macros;
use env_logger;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;

#[derive(Clap, Debug, Clone, Serialize, Deserialize, EnumString, Display)]
enum OptParseType {
    File,
    Tree,
    ApiType,
    ApiMessage,
}

/// Ingest the VPP API JSON definition file and output the Rust code
#[clap(version = "0.1", author = "Andrew Yourtchenko <ayourtch@gmail.com>")]
#[derive(Clap, Debug, Clone, Serialize, Deserialize)]
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

    /// Print message names
    #[clap(long)]
    print_message_names: bool,

    /// Generate the code
    #[clap(long)]
    generate_code: bool,

    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

#[derive(Debug, Clone)]
struct VppJsApiType {
    type_name: String,
    fields: Vec<VppJsApiMessageFieldDef>,
}

impl Serialize for VppJsApiType {
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

struct VppJsApiTypeVisitor;

impl<'de> Visitor<'de> for VppJsApiTypeVisitor {
    type Value = VppJsApiType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct VppJsApiType")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<VppJsApiType, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let type_name: String = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let mut fields: Vec<VppJsApiMessageFieldDef> = vec![];
        loop {
            let nxt = seq.next_element();
            log::debug!("Next: {:#?}", &nxt);
            if let Ok(Some(v)) = nxt {
                fields.push(v);
            } else {
                break;
            }
        }
        Ok(VppJsApiType { type_name, fields })
    }
}

impl<'de> Deserialize<'de> for VppJsApiType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(VppJsApiTypeVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum VppJsApiDefaultValue {
    Str(String),
    Bool(bool),
    I64(i64),
    F64(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VppJsApiFieldOptions {
    #[serde(default)]
    default: Option<VppJsApiDefaultValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum VppJsApiFieldSize {
    Fixed(usize),
    Variable(Option<String>),
}

#[derive(Debug, Clone)]
struct VppJsApiMessageFieldDef {
    ctype: String,
    name: String,
    maybe_size: Option<VppJsApiFieldSize>,
    maybe_options: Option<VppJsApiFieldOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum VppJsApiMessageFieldHelper {
    Str(String),
    Usize(usize),
    Map(VppJsApiFieldOptions),
}

impl Serialize for VppJsApiMessageFieldDef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use crate::VppJsApiFieldSize::*;

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

struct VppJsApiMessageFieldDefVisitor;

impl<'de> Visitor<'de> for VppJsApiMessageFieldDefVisitor {
    type Value = VppJsApiMessageFieldDef;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct VppJsApiMessageField")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<VppJsApiMessageFieldDef, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let ctype: String = if let Some(VppJsApiMessageFieldHelper::Str(s)) = seq.next_element()? {
            s
        } else {
            panic!("Error");
        };
        let name: String = if let Some(VppJsApiMessageFieldHelper::Str(s)) = seq.next_element()? {
            s
        } else {
            panic!("Error 2");
        };

        let mut maybe_sz: Option<usize> = None;
        let mut maybe_sz_name: Option<String> = None;
        let mut maybe_options: Option<VppJsApiFieldOptions> = None;

        loop {
            let nxt = seq.next_element();
            match nxt? {
                Some(VppJsApiMessageFieldHelper::Map(m)) => {
                    maybe_options = Some(m);
                    break;
                }
                Some(VppJsApiMessageFieldHelper::Str(o)) => {
                    maybe_sz_name = Some(o);
                }
                Some(VppJsApiMessageFieldHelper::Usize(o)) => {
                    maybe_sz = Some(o);
                }
                None => break,
            }
        }
        let maybe_size = match (maybe_sz, maybe_sz_name) {
            (None, None) => None,
            (Some(0), None) => Some(VppJsApiFieldSize::Variable(None)),
            (Some(0), Some(s)) => Some(VppJsApiFieldSize::Variable(Some(s))),
            (Some(x), None) => Some(VppJsApiFieldSize::Fixed(x)),
            (None, Some(s)) => panic!("Unexpected dependent field {} with no length", s),
            (Some(x), Some(s)) => panic!("Unexpected dependent field {} with length {}", s, x),
        };
        let ret = VppJsApiMessageFieldDef {
            ctype,
            name,
            maybe_size,
            maybe_options,
        };
        Ok(ret)
    }
}

impl<'de> Deserialize<'de> for VppJsApiMessageFieldDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(VppJsApiMessageFieldDefVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VppJsApiMessageInfo {
    crc: String,
}

#[derive(Debug, Clone)]
struct VppJsApiMessage {
    name: String,
    fields: Vec<VppJsApiMessageFieldDef>,
    info: VppJsApiMessageInfo,
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
enum VppJsApiMessageHelper {
    Field(VppJsApiMessageFieldDef),
    Info(VppJsApiMessageInfo),
    Name(String),
}

struct VppJsApiMessageVisitor;

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

#[derive(Debug, Deserialize)]
struct VppJsApiAlias {
    #[serde(rename = "type")]
    ctype: String,
    length: Option<usize>,
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

#[derive(Debug, Deserialize)]
struct VppJsApiService {
    #[serde(default)]
    events: Vec<String>,
    reply: String,
    stream: Option<bool>,
    #[serde(default)]
    stream_msg: Option<String>,
}

impl Serialize for VppJsApiService {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 1;
        if self.stream.is_some() {
            len = len + 1;
        }
        if self.events.len() > 0 {
            len = len + 1;
        }
        if self.stream_msg.is_some() {
            len = len + 1;
        }
        let mut map = serializer.serialize_map(Some(len))?;
        if self.events.len() > 0 {
            map.serialize_entry("events", &self.events);
        }
        map.serialize_entry("reply", &self.reply)?;
        if let Some(s) = &self.stream {
            map.serialize_entry("stream", s);
        }
        if let Some(s) = &self.stream_msg {
            map.serialize_entry("stream_msg", s);
        }
        map.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VppJsApiOptions {
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VppJsApiEnumInfo {
    enumtype: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct VppJsApiEnumValueDef {
    name: String,
    value: i64,
}

impl Serialize for VppJsApiEnumValueDef {
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
struct VppJsApiEnum {
    name: String,
    values: Vec<VppJsApiEnumValueDef>,
    info: VppJsApiEnumInfo,
}

impl Serialize for VppJsApiEnum {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum VppJsApiEnumHelper {
    Str(String),
    Val(VppJsApiEnumValueDef),
    Map(VppJsApiEnumInfo),
}

struct VppJsApiEnumVisitor;

impl<'de> Visitor<'de> for VppJsApiEnumVisitor {
    type Value = VppJsApiEnum;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct VppJsApiEnum")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<VppJsApiEnum, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let name: String = if let Some(VppJsApiEnumHelper::Str(s)) = seq.next_element()? {
            s
        } else {
            panic!("Error");
        };
        log::debug!("API message: {}", &name);
        let mut values: Vec<VppJsApiEnumValueDef> = vec![];
        let mut maybe_info: Option<VppJsApiEnumInfo> = None;
        loop {
            let nxt = seq.next_element();
            log::debug!("Next: {:#?}", &nxt);
            match nxt? {
                Some(VppJsApiEnumHelper::Val(f)) => values.push(f),
                Some(VppJsApiEnumHelper::Map(i)) => {
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
        Ok(VppJsApiEnum { name, values, info })
    }
}

impl<'de> Deserialize<'de> for VppJsApiEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(VppJsApiEnumVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VppJsApiCounterElement {
    name: String,
    severity: String,
    #[serde(rename = "type")]
    typ: String,
    units: String,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VppJsApiCounter {
    name: String,
    elements: Vec<VppJsApiCounterElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VppJsApiPath {
    path: String,
    counter: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VppJsApiFile {
    types: Vec<VppJsApiType>,
    messages: Vec<VppJsApiMessage>,
    unions: Vec<VppJsApiType>,
    enums: Vec<VppJsApiEnum>,
    #[serde(default)]
    enumflags: Vec<VppJsApiEnum>,
    services: LinkedHashMap<String, VppJsApiService>,
    options: VppJsApiOptions,
    aliases: LinkedHashMap<String, VppJsApiAlias>,
    vl_api_version: String,
    imports: Vec<String>,
    counters: Vec<VppJsApiCounter>,
    paths: Vec<Vec<VppJsApiPath>>,
}

impl VppJsApiFile {
    pub fn verify_data(data: &str, jaf: &VppJsApiFile) {
        use serde_json::Value;
        /*
         * Here we verify that we are not dropping anything during the
         * serialization/deserialization. To do that we use the typeless
         * serde:
         *
         * string_data -> json_deserialize -> json_serialize_pretty -> good_json
         *
         * string_data -> VPPJAF_deserialize -> VPPJAF_serialize ->
         *             -> json_deserialize -> json_serialize_pretty -> test_json
         *
         * Then we compare the two values for being identical and panic if they
         * aren't.
         */

        let good_val: Value = serde_json::from_str(data).unwrap();
        let good_json = serde_json::to_string_pretty(&good_val).unwrap();

        let jaf_serialized_json = serde_json::to_string_pretty(jaf).unwrap();
        let test_val: Value = serde_json::from_str(&jaf_serialized_json).unwrap();
        let test_json = serde_json::to_string_pretty(&test_val).unwrap();

        if good_json != test_json {
            eprintln!("{}", good_json);
            println!("{}", test_json);
            panic!("Different javascript in internal sanity self-test");
        }
    }

    pub fn from_str(data: &str) -> std::result::Result<VppJsApiFile, serde_json::Error> {
        use serde_json::Value;
        let res = serde_json::from_str::<VppJsApiFile>(&data);
        res
    }
}

fn parse_api_tree(opts: &Opts, root: &str, map: &mut LinkedHashMap<String, VppJsApiFile>) {
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
                let desc = VppJsApiFile::from_str(&data);
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

fn get_rust_type_from_ctype(opts: &Opts, ctype: &str) -> String {
    use convert_case::{Case, Casing};

    let result: String = if ctype.starts_with("vl_api_") {
        let ctype_trimmed = ctype.trim_left_matches("vl_api_").trim_right_matches("_t");
        ctype_trimmed.to_case(Case::UpperCamel)
    } else {
        format!("{}", ctype)
    };
    result
}

fn get_rust_field_type(opts: &Opts, fld: &VppJsApiMessageFieldDef, is_last: bool) -> String {
    use crate::VppJsApiFieldSize::*;
    let rtype = get_rust_type_from_ctype(opts, &fld.ctype);
    let full_rtype = if let Some(size) = &fld.maybe_size {
        match size {
            Variable(max_var) => {
                if fld.ctype == "string" {
                    format!("VariableSizeString")
                } else {
                    format!("VariableSizeArray<{}>", rtype)
                }
            }
            Fixed(maxsz) => {
                if fld.ctype == "string" {
                    format!("FixedSizeString<typenum::U{}>", maxsz)
                } else {
                    format!("[{}; {}]", rtype, maxsz)
                }
            }
        }
    } else {
        format!("{}", rtype)
    };
    if fld.maybe_options.is_none() {
        format!("{},", full_rtype)
    } else {
        format!("{}, // {:?} {}", full_rtype, fld, is_last)
    }
}

fn generate_code(opts: &Opts, api_files: &LinkedHashMap<String, VppJsApiFile>) {
    use convert_case::{Case, Casing};
    let mut type_generated: HashMap<String, ()> = HashMap::new();

    let mut acc: String = format!("/* Autogenerated data. Do not edit */\n");

    for (name, f) in api_files {
        acc.push_str(&format!("\n/******** {} types *********/\n\n", &name));
        for m in &f.types {
            let camel_case_name: String = m.type_name.to_case(Case::UpperCamel);
            if type_generated.get(&camel_case_name).is_some() {
                continue;
            }
            acc.push_str(&format!(
                "#[derive(Debug, Clone, Serialize, Deserialize)]\n"
            ));
            acc.push_str(&format!("pub struct {} {{\n", &camel_case_name));
            for (i, fld) in m.fields.clone().into_iter().enumerate() {
                if fld.name == "_vl_msg_id" {
                    continue;
                }
                acc.push_str(&format!(
                    "    pub {}: {}\n",
                    &fld.name,
                    get_rust_field_type(opts, &fld, i == m.fields.len() - 1)
                ));
            }
            acc.push_str(&format!("}}\n\n"));
            type_generated.insert(camel_case_name, ());
        }

        acc.push_str(&format!("\n/******** {} messages *********/\n\n", &name));

        for m in &f.messages {
            let crc = &m.info.crc.strip_prefix("0x").unwrap();
            let camel_case_name: String = m.name.to_case(Case::UpperCamel);
            if type_generated.get(&camel_case_name).is_some() {
                continue;
            }
            acc.push_str(&format!(
                "#[derive(Debug, Clone, Serialize, Deserialize)]\n"
            ));
            acc.push_str(&format!("pub struct {} {{\n", &camel_case_name));
            for (i, fld) in m.fields.clone().into_iter().enumerate() {
                if fld.name == "_vl_msg_id" {
                    continue;
                }
                acc.push_str(&format!(
                    "    pub {}: {}\n",
                    &fld.name,
                    get_rust_field_type(opts, &fld, i == m.fields.len() - 1)
                ));
            }
            acc.push_str(&format!("}}\n\n"));
            type_generated.insert(camel_case_name, ());

            // println!("{}_{}", &m.name, &crc);
        }
    }
    println!("{}", acc);
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
                let desc = VppJsApiFile::from_str(&data).unwrap();
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
                let desc: VppJsApiType = serde_json::from_str(&data).unwrap();
                println!("Dump Type: {:#?}", &desc);
            }
            OptParseType::ApiMessage => {
                let desc: VppJsApiMessage = serde_json::from_str(&data).unwrap();
                println!("Dump: {:#?}", &desc);
            }
        }
    } else {
        match opts.parse_type {
            OptParseType::Tree => {
                // it was a directory tree, descend downwards...
                let mut api_files: LinkedHashMap<String, VppJsApiFile> = LinkedHashMap::new();
                parse_api_tree(&opts, &opts.in_file, &mut api_files);
                println!("// Loaded {} API definition files", api_files.len());
                if opts.print_message_names {
                    for (name, f) in &api_files {
                        for m in &f.messages {
                            let crc = &m.info.crc.strip_prefix("0x").unwrap();
                            println!("{}_{}", &m.name, &crc);
                        }
                    }
                }
                if opts.generate_code {
                    generate_code(&opts, &api_files);
                }
            }
            e => {
                panic!("inappropriate parse type {:?} for inexistent file", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_data_path() -> PathBuf {
        let mut path = PathBuf::from(file!());
        path.pop();
        path.pop();
        path.pop();
        path.push("testdata/vpp/");
        path
    }

    fn parse_api_tree_with_verify(root: &str, map: &mut LinkedHashMap<String, VppJsApiFile>) {
        use std::fs;
        for entry in fs::read_dir(root).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            let metadata = fs::metadata(&path).unwrap();
            if metadata.is_file() {
                let res = std::fs::read_to_string(&path);
                if let Ok(data) = res {
                    let desc = VppJsApiFile::from_str(&data);
                    if let Ok(d) = desc {
                        VppJsApiFile::verify_data(&data, &d);
                        map.insert(path.to_str().unwrap().to_string(), d);
                    } else {
                        eprintln!("Error loading {:?}: {:?}", &path, &desc);
                    }
                } else {
                    eprintln!("Error reading {:?}: {:?}", &path, &res);
                }
            }
            if metadata.is_dir() && entry.file_name() != "." && entry.file_name() != ".." {
                parse_api_tree_with_verify(&path.to_str().unwrap(), map);
            }
        }
    }

    #[test]
    fn test_tree() {
        let mut api_files: LinkedHashMap<String, VppJsApiFile> = LinkedHashMap::new();
        parse_api_tree_with_verify(get_test_data_path().to_str().unwrap(), &mut api_files);

        assert_eq!(123, api_files.len());
    }
}
