#![allow(
    dead_code,
    unused_mut,
    unused_variables,
    unused_must_use,
    non_camel_case_types,
    unused_imports
)]
use serde::{Deserialize, Serialize};
extern crate strum;
use crate::alias::VppJsApiAlias;
use crate::enums::VppJsApiEnum;
use crate::message::VppJsApiMessage;
use crate::services::{VppJsApiOptions, VppJsApiService};
use crate::types::VppJsApiType;
use linked_hash_map::LinkedHashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VppJsApiCounterElement {
    pub name: String,
    pub severity: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub units: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VppJsApiCounter {
    pub name: String,
    pub elements: Vec<VppJsApiCounterElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VppJsApiPath {
    pub path: String,
    pub counter: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VppJsApiFile {
    pub types: Vec<VppJsApiType>,
    pub messages: Vec<VppJsApiMessage>,
    pub unions: Vec<VppJsApiType>,
    pub enums: Vec<VppJsApiEnum>,
    #[serde(default)]
    pub enumflags: Vec<VppJsApiEnum>,
    pub services: LinkedHashMap<String, VppJsApiService>,
    pub options: VppJsApiOptions,
    pub aliases: LinkedHashMap<String, VppJsApiAlias>,
    pub vl_api_version: String,
    pub imports: Vec<String>,
    pub counters: Vec<VppJsApiCounter>,
    pub paths: Vec<Vec<VppJsApiPath>>,
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
        // use serde_json::Value;
        let res = serde_json::from_str::<VppJsApiFile>(&data);
        res
    }
}
