use clap::Clap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

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

    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct VppApiFieldDef {
    ctype: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct VppApiType {
    type_name: String,
    fields: Vec<VppApiFieldDef>,
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
        let mut fields: Vec<VppApiFieldDef> = vec![];
        loop {
            if let Ok(Some(v)) = seq.next_element() {
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
struct VppApiFile {
    types: Vec<VppApiType>,
}

fn main() {
    let opts: Opts = Opts::parse();

    if let Ok(data) = std::fs::read_to_string(&opts.in_file) {
        let desc: VppApiFile = serde_json::from_str(&data).unwrap();
        println!("File: {:#?}", &desc);
    }
}
