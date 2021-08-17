use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};
extern crate strum;
use crate::basetypes::{maxSizeUnion, sizeof_alias, sizeof_struct};
use crate::parser_helper::{camelize_ident, get_ident, get_type};
use serde::de::{Deserializer, SeqAccess, Visitor};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VppJsApiEnumInfo {
    pub enumtype: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VppJsApiEnumValueDef {
    pub name: String,
    pub value: i64,
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

#[derive(Debug, Clone)]
pub struct VppJsApiEnum {
    pub name: String,
    pub values: Vec<VppJsApiEnumValueDef>,
    pub info: VppJsApiEnumInfo,
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
pub enum VppJsApiEnumHelper {
    Str(String),
    Val(VppJsApiEnumValueDef),
    Map(VppJsApiEnumInfo),
}

pub struct VppJsApiEnumVisitor;

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
impl VppJsApiEnum {
    pub fn generate_code(&self) -> String {
        let mut code = String::new();
        code.push_str(&format!(
            "#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)] \n"
        ));
        match &self.info.enumtype {
            Some(len) => code.push_str(&format!("#[repr({})]\n", &len)),
            _ => code.push_str(&format!("#[repr(u32)]\n")),
        }
        code.push_str(&format!("pub enum {} {{ \n", camelize_ident(&self.name)));
        for x in 0..self.values.len() {
            code.push_str(&format!(
                "\t {}={}, \n",
                get_ident(&self.values[x].name),
                self.values[x].value
            ));
        }
        // code.push_str("\t #[serde(other)] \n\t Invalid \n");
        code.push_str("} \n");
        code
    }
    pub fn iter_and_generate_code(
        enums: &Vec<VppJsApiEnum>,
        api_definition: &mut Vec<(String, String)>,
        name: &str,
    ) -> String {
        enums
            .iter()
            .filter(|x| {
                for j in 0..api_definition.len() {
                    if &api_definition[j].0 == &x.name {
                        return false;
                    }
                }
                api_definition.push((x.name.clone(), name.to_string().clone()));
                return true;
            })
            .fold(String::new(), |mut acc, x| {
                acc.push_str(&x.generate_code());
                acc
            })
    }
}
