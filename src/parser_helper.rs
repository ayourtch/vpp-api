use std::string::ToString;
extern crate strum;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use crate::file_schema::*;
use crate::Opts;
use crate::types::*;


pub fn parse_api_tree(opts: &Opts, root: &str, map: &mut LinkedHashMap<String, VppJsApiFile>) {
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
pub fn get_type(apitype: &str) -> String {
    let rtype = if apitype.starts_with("vl_api_"){
        let ctype_trimmed = apitype.trim_start_matches("vl_api_").trim_end_matches("_t");
        // String::from(ctype_trimmed)
        camelize_ident(ctype_trimmed)
    } else {
            if apitype == "string" {
                format!("String")
            }
            else {
                format!("{}", apitype)
            }
    };
    rtype
}
pub fn get_ident(api_ident: &str) -> String {
    if api_ident == "type" {
        format!("typ")
    }
    else {
        format!("{}",api_ident.trim_start_matches("_"))
    }
}

pub fn get_rust_type_from_ctype(
    _opts: &Opts,
    enum_containers: &HashMap<String, String>,
    ctype: &str,
) -> String {
    use convert_case::{Case, Casing};

    let rtype = {
        let rtype: String = if ctype.starts_with("vl_api_") {
            let ctype_trimmed = ctype.trim_start_matches("vl_api_").trim_end_matches("_t");
            ctype_trimmed.to_case(Case::UpperCamel)
        } else {
            format!("{}", ctype)
        };
        /* if the candidate Rust type is an enum, we need to create
        a parametrized type such that we knew which size to
        deal with at serialization/deserialization time */

        if let Some(container) = enum_containers.get(&rtype) {
            format!("SizedEnum<{}, {}>", rtype, container)
        } else {
            rtype
        }
    };
    rtype
}

pub fn get_rust_field_name(opts: &Opts, name: &str) -> String {
    if name == "type" || name == "match" {
        format!("r#{}", name)
    } else {
        format!("{}", name)
    }
}

pub fn get_rust_field_type(
    opts: &Opts,
    enum_containers: &HashMap<String, String>,
    fld: &VppJsApiMessageFieldDef,
    is_last: bool,
) -> String {
    use crate::VppJsApiFieldSize::*;
    let rtype = get_rust_type_from_ctype(opts, enum_containers, &fld.ctype);
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
                    format!("FixedSizeArray<{}, typenum::U{}>", rtype, maxsz)
                }
            }
        }
    } else {
        format!("{}", rtype)
    };
    if fld.maybe_options.is_none() {
        format!("{}", full_rtype)
    } else {
        format!("{} /* {:?} {} */", full_rtype, fld, is_last)
    }
}

pub fn camelize_ident(ident:&str) -> String {
    let mut c = ident.split("_");
    let collection: Vec<&str> = c.collect();
    let mut finalString = String::new();

    for x in collection{
        for (i, c) in x.chars().enumerate() {
            if i==0 {
                let c_upper: Vec<_> = c.to_uppercase().collect();
                finalString.push_str(&c_upper[0].to_string());
            }
            else {
                finalString.push_str(&c.to_string());
            }
        }
    }
    finalString
}

pub fn camelize(opts: &Opts, ident: &str) -> String {
    use convert_case::{Case, Casing};
    ident.to_case(Case::UpperCamel)
}

#[derive(Clone, Default, Debug)]
pub struct GeneratedType {
    pub derives: LinkedHashMap<String, ()>,
    pub file: String,
    pub text: String,
}

impl GeneratedType {
    pub fn add_derives(self: &mut Self, derives: Vec<&str>) {
        for d in derives {
            self.derives.insert(d.to_string(), ());
        }
    }

    pub fn push_str(self: &mut Self, data: &str) {
        self.text.push_str(data);
    }
}

pub fn generate_code(opts: &Opts, api_files: &LinkedHashMap<String, VppJsApiFile>) {
    let mut type_needs_copy_trait: HashMap<String, ()> = HashMap::new();
    let mut enum_containers: HashMap<String, String> = HashMap::new();

    let mut type_generated: LinkedHashMap<String, GeneratedType> = LinkedHashMap::new();

    let mut preamble: String = format!("/* Autogenerated data. Do not edit */\n");
    preamble.push_str("#![allow(non_camel_case_types)]\n");
    preamble.push_str("use serde::{de::DeserializeOwned, Deserialize, Serialize};\n");
    preamble.push_str("use vpp_api_encoding::typ::*;\n");
    preamble.push_str("use vpp_api_transport::*;\n");

    for (name, f) in api_files {
        for m in &f.unions {
            let camel_case_name = camelize(opts, &m.type_name);
            if type_generated.get(&camel_case_name).is_some() {
                continue;
            }
            let mut acc = GeneratedType {
                file: name.clone(),
                ..Default::default()
            };

            /* put the RealUnion type that is a union and is private */

            acc.add_derives(vec!["Copy", "Clone"]);
            acc.push_str(&format!("union TrueUnion{} {{\n", &camel_case_name));
            for (i, fld) in m.fields.clone().into_iter().enumerate() {
                if fld.name == "_vl_msg_id" {
                    continue;
                }
                let field_type =
                    get_rust_field_type(opts, &enum_containers, &fld, i == m.fields.len() - 1);
                /* FIXME: This is very hacky... Special-case the "difficult" types so we don't have
                 * to propagate the "copy" too far... 
                 *
                 * It looks like using the union to determine the max size is not a good idea after
                 * all...
                 */
                let field_type = match (field_type.as_ref()) {
                    "Prefix" => "CopyPrefix".to_string(),
                    "Ip4Address" => "CopyIp4Address".to_string(),
                    "Ip6Address" => "CopyIp6Address".to_string(),
                    "MacAddress" => "CopyMacAddress".to_string(),
                    x => x.to_string(),
                };
                type_needs_copy_trait.insert(field_type.clone(), ());
                acc.push_str(&format!(
                    "    {}: {},\n",
                    get_rust_field_name(opts, &fld.name),
                    field_type,
                ));
            }
            acc.push_str(&format!("}}\n\n"));
            type_generated.insert(format!("TrueUnion{}", &camel_case_name), acc);

            /* now put the tagged enum type that is what really wanted */
            let mut acc = GeneratedType {
                file: name.clone(),
                ..Default::default()
            };

            acc.add_derives(vec!["Debug", "Clone", "Serialize", "Deserialize"]);
            acc.push_str(&format!("pub enum {} {{\n", &camel_case_name));
            for (i, fld) in m.fields.clone().into_iter().enumerate() {
                if fld.name == "_vl_msg_id" {
                    continue;
                }
                let camel_fldname = camelize(opts, &fld.name);
                acc.push_str(&format!(
                    "    {}({}),\n",
                    get_rust_field_name(opts, &camel_fldname),
                    get_rust_field_type(opts, &enum_containers, &fld, i == m.fields.len() - 1)
                ));
            }
            /* add a stash to store the bytes that are not yet parsed */
            acc.push_str(&format!(
                "    UnparsedBytes([u8; std::mem::size_of::<TrueUnion{}>()]),\n",
                &camel_case_name
            ));
            acc.push_str(&format!("}}\n\n"));

            acc.push_str(&format!(
                "impl Default for {} {{ fn default() -> Self {{ Self::UnparsedBytes(Default::default())  }} }}\n\n",
                &camel_case_name,
            ));
            type_generated.insert(camel_case_name, acc);
        }

        for (mname, m) in &f.aliases {
            let camel_case_name = camelize(opts, &mname);
            if type_generated.get(&camel_case_name).is_some() {
                continue;
            }
            let mut acc = GeneratedType {
                file: name.clone(),
                ..Default::default()
            };
            acc.add_derives(vec!["Debug", "Clone", "Serialize", "Deserialize"]);

            let need_copy_trait = type_needs_copy_trait.get(&camel_case_name).is_some();
            if need_copy_trait {
                acc.add_derives(vec!["Copy"]);
            }

            let rtype = get_rust_type_from_ctype(opts, &enum_containers, &m.ctype);
            let rtype = if let Some(sz) = m.length {
                format!("FixedSizeArray<{}, typenum::U{}>", rtype, sz)
            } else {
                rtype
            };

            if need_copy_trait {
                type_generated
                    .entry(rtype.clone())
                    .or_insert(GeneratedType {
                        ..Default::default()
                    })
                    .add_derives(vec!["Copy"]);
            }

            acc.push_str(&format!(
                "pub struct {} (pub {});\n",
                &camel_case_name, rtype
            ));

            acc.push_str(&format!("/* {:#?} */\n\n", &m));
            type_generated.insert(camel_case_name, acc);
        }

        for m in &f.enums {
            let camel_case_name = camelize(opts, &m.name);
            if type_generated.get(&camel_case_name).is_some() {
                continue;
            }

            let v0chars: &Vec<char> = &m.values[0].name.chars().collect();
            let mut value_prefix_len = if m.values.len() <= 1 {
                0 /* a single string does not have a common prefix */
            } else {
                if let Some(pos) = &m.values[0].name.rfind('_') {
                    *pos + 1
                } else {
                    0
                }
            };
            for v in &m.values {
                let vXchars: Vec<char> = v.name.chars().collect();
                if vXchars.len() < value_prefix_len {
                    /* FIXME: we should really set this to be length
                     * minus characters to the first underscore
                     */
                    value_prefix_len = vXchars.len() - 1;
                }
                for i in 0..value_prefix_len {
                    if vXchars[i] != v0chars[i] {
                        value_prefix_len = i;
                        break;
                    }
                }
            }

            let mut acc = GeneratedType {
                file: name.clone(),
                ..Default::default()
            };
            let enum_container_type = m.info.enumtype.clone().unwrap();

            acc.add_derives(vec![
                "Debug",
                "Clone",
                "Copy",
                "Serialize",
                "Deserialize",
                "Eq",
                "PartialEq",
            ]);

            acc.push_str(&format!("pub enum {} {{\n", &camel_case_name));
            acc.push_str(&format!("    // Size: {}\n", &enum_container_type));

            let mut first_value: Option<String> = None;
            for v in &m.values {
                let short_name = &v.name[value_prefix_len..];
                let name_prefix = if short_name.chars().nth(0).unwrap().is_ascii_alphabetic() {
                    format!("")
                } else {
                    format!("x")
                };

                acc.push_str(&format!(
                    "    // {} = {}\n    {}{} = {},\n",
                    &v.name, &v.value, &name_prefix, &short_name, v.value
                ));
                if first_value.is_none() {
                    first_value = Some(format!("{}{}", &name_prefix, &short_name));
                }
            }
            acc.push_str(&format!("}}\n\n"));

            acc.push_str(&format!(
                "impl Default for {} {{ fn default() -> Self {{ Self::{} }} }}\n\n",
                &camel_case_name,
                first_value.unwrap()
            ));

            acc.push_str(&format!(
                "impl AsU32 for {} {{ fn as_u32(data: Self) -> u32 {{ data as u32 }} }}\n\n",
                &camel_case_name
            ));

            enum_containers.insert(camel_case_name.clone(), enum_container_type);
            type_generated.insert(camel_case_name, acc);
        }

        for m in &f.types {
            let camel_case_name = camelize(opts, &m.type_name);
            if type_generated.get(&camel_case_name).is_some() {
                continue;
            }
            let mut acc = type_generated
                .entry(camel_case_name.clone())
                .or_insert(GeneratedType {
                    ..Default::default()
                });
            acc.file = name.clone();

            acc.add_derives(vec![
                "Debug",
                "Default",
                "Clone",
                "Serialize",
                "Deserialize",
            ]);
            let need_copy_trait = type_needs_copy_trait.get(&camel_case_name).is_some();
            if need_copy_trait {
                acc.add_derives(vec!["Copy"]);
            }
            acc.push_str(&format!("pub struct {} {{\n", &camel_case_name));

            let mut copy_types: LinkedHashMap<String, ()> = LinkedHashMap::new();

            for (i, fld) in m.fields.clone().into_iter().enumerate() {
                if fld.name == "_vl_msg_id" {
                    continue;
                }
                let type_name =
                    get_rust_field_type(opts, &enum_containers, &fld, i == m.fields.len() - 1);
                if need_copy_trait {
                    copy_types.insert(type_name.clone(), ());
                }
                acc.push_str(&format!(
                    "    pub {}: {},\n",
                    get_rust_field_name(opts, &fld.name),
                    type_name
                ));
            }
            acc.push_str(&format!("}}\n\n"));
            let acc = ();
            for (k, _) in copy_types {
                type_generated
                    .entry(k)
                    .or_insert(GeneratedType {
                        ..Default::default()
                    })
                    .add_derives(vec!["Copy"]);
            }
        }

        for m in &f.messages {
            let crc = &m.info.crc.strip_prefix("0x").unwrap();
            let camel_case_name = camelize(opts, &m.name);
            if type_generated.get(&camel_case_name).is_some() {
                continue;
            }
            let mut acc = GeneratedType {
                file: name.clone(),
                ..Default::default()
            };
            acc.add_derives(vec![
                "Debug",
                "Default",
                "Clone",
                "Serialize",
                "Deserialize",
            ]);
            let need_copy_trait = type_needs_copy_trait.get(&camel_case_name).is_some();
            if need_copy_trait {
                acc.add_derives(vec!["Copy"]);
            }
            acc.push_str(&format!("pub struct {} {{\n", &camel_case_name));
            for (i, fld) in m.fields.clone().into_iter().enumerate() {
                if fld.name == "_vl_msg_id" {
                    continue;
                }
                acc.push_str(&format!(
                    "    pub {}: {},\n",
                    get_rust_field_name(opts, &fld.name),
                    get_rust_field_type(opts, &enum_containers, &fld, i == m.fields.len() - 1)
                ));
            }
            acc.push_str(&format!("}}\n\n"));
            type_generated.insert(camel_case_name, acc);

            // println!("{}_{}", &m.name, &crc);
        }
    }

    println!("{}\n", preamble);
    for (aname, adata) in type_generated {
        let derives = adata
            .derives
            .keys()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        if adata.text == "" {
            println!("/* #[derive({})] // auto {} */", derives, &aname);
        } else {
            println!("#[derive({})] // auto {}", derives, &aname);
            println!("{}", adata.text);
            println!("");
        }
    }
}