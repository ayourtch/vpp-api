use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};
extern crate strum;
use crate::basetypes::{maxSizeUnion, sizeof_alias, sizeof_struct, field_size};
use crate::file_schema::VppJsApiFile;
use crate::parser_helper::{camelize_ident, get_ident, get_type};
use linked_hash_map::LinkedHashMap;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt;

// This holds the Type and Union Data
#[derive(Debug, Clone)]
pub struct VppJsApiType {
    pub type_name: String,
    pub fields: Vec<VppJsApiMessageFieldDef>,
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

pub struct VppJsApiTypeVisitor;

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
impl VppJsApiType {
    pub fn generate_code(&self) -> String {
        let mut code = String::new();
        code.push_str(&format!("// Implementation for {} \n", &self.type_name));
        code.push_str(&format!(
            "#[derive(Debug, Clone, Serialize, Deserialize, Default)] \n"
        ));
        code.push_str(&format!(
            "pub struct {} {{ \n",
            camelize_ident(&self.type_name)
        ));
        for x in 0..self.fields.len() {
            // println!("{:#?}", self.fields);
            code.push_str(&format!("\tpub {} : ", get_ident(&self.fields[x].name)));
            if self.fields[x].ctype == "string" {
                match &self.fields[x].maybe_size {
                    Some(cont) => match cont {
                        VppJsApiFieldSize::Fixed(len) => {
                            code.push_str(&format!("FixedSizeString<typenum::U{}>, \n", len))
                        }
                        VppJsApiFieldSize::Variable(None) => {
                            code.push_str(&format!("VariableSizeString, \n"))
                        }
                        _ => code.push_str(&format!("{},\n", get_ident(&self.fields[x].name))),
                    },
                    _ => code.push_str(&format!("{}, \n", get_ident(&self.fields[x].name))),
                }
            } 
            else if self.fields[x].ctype.contains("flag"){
                code.push_str(&format!("EnumFlag<{}>, \n",get_type(&self.fields[x].ctype) ));
            }
            else {
                match &self.fields[x].maybe_size {
                    Some(cont) => match cont {
                        VppJsApiFieldSize::Fixed(len) => code.push_str(&format!(
                            "FixedSizeArray<{}, typenum::U{}>, \n",
                            get_type(&self.fields[x].ctype),
                            len
                        )),
                        VppJsApiFieldSize::Variable(t) => {
                            code.push_str(&format!("VariableSizeArray<{}>, \n", get_type(&self.fields[x].ctype)))
                        }
                        _ => code.push_str(&format!("{},\n", get_type(&self.fields[x].ctype))),
                    },
                    _ => code.push_str(&format!("{}, \n", get_type(&self.fields[x].ctype))),
                }
            }
        }
        code.push_str("} \n");
        code
    }
    pub fn generate_code_union(&self, apifile: &VppJsApiFile) -> String {
        let mut code = String::new();
        code.push_str(&format!(
            "#[derive(Debug, Clone, Serialize, Deserialize, Default, VppUnionIdent)] \n"
        ));
        for x in 0..self.fields.len(){
            let size_of_typ = field_size(&self.fields[x], &apifile);
            let ident =  get_type(&self.fields[x].ctype);
            code.push_str(&format!("#[types({}:{})] \n",ident, size_of_typ));
        }
        let unionsize = maxSizeUnion(&self, &apifile);
        code.push_str(&format!(
            "pub struct {}(FixedSizeArray<u8, typenum::U{}>); \n",
            camelize_ident(&self.type_name),
            unionsize
        ));
        code
    }
    pub fn iter_and_generate_code(
        structs: &Vec<VppJsApiType>,
        api_definition: &mut Vec<(String, String)>,
        name: &str,
        import_table: &mut Vec<(String, Vec<String>)>,
    ) -> String {
        structs
            .iter()
            .filter(|x| {
                for j in 0..api_definition.len() {
                    if &api_definition[j].0 == &x.type_name {
                        for k in 0..import_table.len() {
                            if &import_table[k].0 == &api_definition[j].1 {
                                if !import_table[k].1.contains(&x.type_name) {
                                    // println!("Pushing");
                                    import_table[k].1.push(x.type_name.clone());
                                    return false;
                                } else {
                                    // println!("Ignoring");
                                    return false;
                                }
                            }
                        }
                        // println!("Contents of api defintion: {}", api_definition[j].1);
                        // println!("pushing into arr {} {}", api_definition[j].1, x.type_name);
                        import_table.push((api_definition[j].1.clone(), vec![x.type_name.clone()]));
                        return false;
                    }
                }
                api_definition.push((x.type_name.clone(), name.to_string().clone()));
                return true;
            })
            .fold(String::new(), |mut acc, x| {
                acc.push_str(&x.generate_code());
                acc
            })
    }
    pub fn iter_and_generate_code_union(
        unions: &Vec<VppJsApiType>,
        api_definition: &mut Vec<(String, String)>,
        name: &str,
        file: &VppJsApiFile,
        import_table: &mut Vec<(String, Vec<String>)>,
    ) -> String {
        unions
            .iter()
            .filter(|x| {
                for j in 0..api_definition.len() {
                    if &api_definition[j].0 == &x.type_name {
                        for k in 0..import_table.len() {
                            if &import_table[k].0 == &api_definition[j].1 {
                                if !import_table[k].1.contains(&x.type_name) {
                                    // println!("Pushing");
                                    import_table[k].1.push(x.type_name.clone());
                                    return false;
                                } else {
                                    // println!("Ignoring");
                                    return false;
                                }
                            }
                        }
                        import_table.push((api_definition[j].1.clone(), vec![x.type_name.clone()]));
                        return false;
                    }
                }
                api_definition.push((x.type_name.clone(), name.to_string().clone()));
                return true;
            })
            .fold(String::new(), |mut acc, x| {
                acc.push_str(&x.generate_code_union(&file));
                acc
            })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VppJsApiDefaultValue {
    Str(String),
    Bool(bool),
    I64(i64),
    F64(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VppJsApiFieldOptions {
    #[serde(default)]
    pub default: Option<VppJsApiDefaultValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VppJsApiFieldSize {
    Fixed(usize),
    Variable(Option<String>),
}

#[derive(Debug, Clone)]
pub struct VppJsApiMessageFieldDef {
    pub ctype: String,
    pub name: String,
    pub maybe_size: Option<VppJsApiFieldSize>,
    pub maybe_options: Option<VppJsApiFieldOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VppJsApiMessageFieldHelper {
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
                Some(Fixed(_n)) => 1,
                Some(Variable(None)) => 1,
                Some(Variable(Some(_x))) => 2,
            };
        let mut seq = serializer.serialize_seq(Some(len))?;
        seq.serialize_element(&self.ctype)?;
        seq.serialize_element(&self.name)?;
        match &self.maybe_size {
            None => { /* do nothing */ }
            Some(Fixed(n)) => {
                seq.serialize_element(&n)?;
            }
            Some(Variable(None)) => {
                seq.serialize_element(&0u32)?;
            }
            Some(Variable(Some(x))) => {
                seq.serialize_element(&0u32)?;
                seq.serialize_element(&x)?;
            }
        }

        if let Some(o) = &self.maybe_options {
            seq.serialize_element(o)?;
        }
        seq.end()
    }
}

pub struct VppJsApiMessageFieldDefVisitor;

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
