use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use crate::basetypes::{maxSizeUnion, sizeof_alias, sizeof_struct};
use crate::parser_helper::{camelize_ident, get_ident, get_type};
use linked_hash_map::LinkedHashMap;
extern crate strum;

#[derive(Debug, Deserialize, Clone)]
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
            map.serialize_entry("length", s)?;
        }
        map.end()
    }
}
impl VppJsApiAlias {
    pub fn generate_code(&self, name: &str) -> String {
        let mut code = String::new();
        code.push_str(&format!("pub type {}=", camelize_ident(&get_ident(&name))));
        match self.length {
            Some(len) => {
                let newtype = get_type(&self.ctype);
                code.push_str(&format!("[{};{}]; \n", newtype, len));
            }
            _ => code.push_str(&format!("{}; \n", get_type(&self.ctype))),
        }
        code
    }
    // Handling Vector of Alias
    pub fn iter_and_generate_code(
        aliases: &LinkedHashMap<String, VppJsApiAlias>,
        api_definition: &mut Vec<(String, String)>,
        name: &str,
    ) -> String {
        aliases
            .keys()
            .filter(|x| {
                for j in 0..api_definition.len() {
                    if &api_definition[j].0 == *x {
                        return false;
                    }
                }
                api_definition.push((x.clone().to_string(), name.to_string().clone()));
                return true;
            })
            .fold(String::new(), |mut acc, x| {
                acc.push_str(&aliases[x].generate_code(x));
                acc
            })
    }
}
