use crate::*;
use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};
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

#[derive(Debug)]
enum MessageType {
    Request,
    Reply,
}

#[derive(Debug)]
enum MessageTypeError {
    ReqMissingFields,
    ReqZerothFieldMustBeMsgId(VppJsApiMessageFieldDef),
    ReqFirstFieldMustBeClientIndex(VppJsApiMessageFieldDef),
    ReqSecondFieldMustBeContext(VppJsApiMessageFieldDef),
    RepMissingFields,
    RepZerothFieldMustBeMsgId(VppJsApiMessageFieldDef),
    RepFirstFieldMustBeContext(VppJsApiMessageFieldDef),
}

impl VppJsApiMessage {
    fn get_message_type(&self, file: &VppJsApiFile) -> Result<MessageType, MessageTypeError> {
        if let Some(svc) = file.services.get(&self.name) {
            if self.fields.len() < 3 {
                return Err(MessageTypeError::ReqMissingFields);
            }
            if self.fields[0].name != "_vl_msg_id" {
                return Err(MessageTypeError::ReqZerothFieldMustBeMsgId(
                    self.fields[0].clone(),
                ));
            }
            if self.fields[1].name != "client_index" {
                return Err(MessageTypeError::ReqFirstFieldMustBeClientIndex(
                    self.fields[1].clone(),
                ));
            }
            if self.fields[2].name != "context" {
                return Err(MessageTypeError::ReqSecondFieldMustBeContext(
                    self.fields[2].clone(),
                ));
            }
            return Ok(MessageType::Request);
        }
        // anything that is not a request is assumed to be a response
        if self.fields.len() < 2 {
            return Err(MessageTypeError::RepMissingFields);
        }
        if self.fields[0].name != "_vl_msg_id" {
            return Err(MessageTypeError::RepZerothFieldMustBeMsgId(
                self.fields[0].clone(),
            ));
        }
        if self.fields[1].name != "context" {
            return Err(MessageTypeError::RepFirstFieldMustBeContext(
                self.fields[1].clone(),
            ));
        }
        return Ok(MessageType::Reply);
    }

    pub fn generate_code(&self, file: &VppJsApiFile) -> String {
        let mut code = String::new();

        let mtype = self.get_message_type(file);
        if let Err(e) = mtype {

            eprintln!(
                "Could not determine message type for message {}, error: {:?}",
                self.name, e
            );
            return "".to_string();
        }
        let mtype = mtype.unwrap();

        let type_specific_derive = match mtype {
            MessageType::Request => {
                format!(", VppMessageRequest")
            }
            MessageType::Reply => {
                format!(", VppMessageReply")
            }
        };

        code.push_str(&format!(
            "#[derive(Debug, Clone, Serialize, Deserialize{})]\n",
            type_specific_derive
        ));
        code.push_str(&format!(
            "#[message_name_and_crc({}_{})]\n",
            self.name,
            self.info.crc.trim_start_matches("0x")
        ));
        code.push_str(&format!("pub struct {} {{\n", camelize_ident(&self.name)));
        for x in 0..self.fields.len() {
            match mtype {
                MessageType::Request => {
                    if x < 3 {
                        continue;
                    }
                }
                MessageType::Reply => {
                    if x < 2 {
                        continue;
                    }
                }
            }
            if self.fields[x].name == "_vl_msg_id" {
                // panic!("Something wrong");
            } else if self.fields[x].ctype == "string" {
                match &self.fields[x].maybe_size {
                    Some(cont) => match cont {
                        VppJsApiFieldSize::Fixed(len) => code.push_str(&format!(
                            "\tpub {}: FixedSizeString<typenum::U{}>,\n",
                            get_ident(&self.fields[x].name),
                            len
                        )),
                        VppJsApiFieldSize::Variable(None) => code.push_str(&format!(
                            "\tpub {}: VariableSizeString,\n",
                            get_ident(&self.fields[x].name)
                        )),
                        _ => code
                            .push_str(&format!("\tpub {}: ,\n", get_ident(&self.fields[x].name))),
                    },
                    _ => code.push_str(&format!("\tpub {}:,\n", get_ident(&self.fields[x].name))),
                }
            } else if self.fields[x].ctype.contains("flag") {
                code.push_str(&format!(
                    "\t pub {}: EnumFlag<{}>,\n",
                    get_ident(&self.fields[x].name),
                    get_type(&self.fields[x].ctype)
                ));
            } else {
                code.push_str(&format!("\tpub {}: ", get_ident(&self.fields[x].name)));
                match &self.fields[x].maybe_size {
                    Some(cont) => match cont {
                        VppJsApiFieldSize::Fixed(len) => code.push_str(&format!(
                            "FixedSizeArray<{}, typenum::U{}>,\n",
                            get_type(&self.fields[x].ctype),
                            len
                        )),
                        VppJsApiFieldSize::Variable(t) => code.push_str(&format!(
                            "VariableSizeArray<{}>,\n",
                            get_type(&self.fields[x].ctype)
                        )),
                    },
                    _ => code.push_str(&format!("{},\n", get_type(&self.fields[x].ctype))),
                    /*code.push_str(&format!(
                        "\tpub {}: {},\n",
                        get_ident(&self.fields[x].name),
                        get_type(&self.fields[x].ctype)
                    ));*/
                }
            }
        }
        code.push_str("}\n");
        // self.gen_impl_messages(&mut code);
        code
    }
    pub fn gen_impl_messages(&self, file: &mut String) {
        file.push_str(&format!("impl {} {{\n", camelize_ident(&self.name)));
        file.push_str(&format!(
            "\t pub fn get_message_name_and_crc() -> String {{\n"
        ));
        file.push_str(&format!(
            "\t \t String::from(\"{}_{}\")\n",
            self.name,
            self.info.crc.trim_start_matches("0x")
        ));
        file.push_str(&format!("\t }}\n"));
        file.push_str(&format!("}}\n"));
    }
    pub fn iter_and_generate_code(file: &VppJsApiFile) -> String {
        file.messages.iter().fold(String::new(), |mut acc, x| {
            acc.push_str(&x.generate_code(file));
            acc
        })
    }
}
