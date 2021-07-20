use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
extern crate strum;

#[derive(Debug, Deserialize, Clone)]
pub struct VppJsApiService {
    #[serde(default)]
    pub events: Vec<String>,
    pub reply: String,
    pub stream: Option<bool>,
    #[serde(default)]
    pub stream_msg: Option<String>,
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
            map.serialize_entry("events", &self.events)?;
        }
        map.serialize_entry("reply", &self.reply)?;
        if let Some(s) = &self.stream {
            map.serialize_entry("stream", s)?;
        }
        if let Some(s) = &self.stream_msg {
            map.serialize_entry("stream_msg", s)?;
        }
        map.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VppJsApiOptions {
    pub version: String,
}
