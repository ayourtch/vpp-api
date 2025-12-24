use std::string::ToString;
extern crate strum;
use crate::file_schema::*;
use crate::types::*;
use crate::Opts;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;

pub fn parse_api_tree(opts: &Opts, root: &str, map: &mut LinkedHashMap<String, VppJsApiFile>) {
    use std::fs;
    if opts.verbose > 2 {
        println!("parse tree: {:?}", root);
    }

    let mut entries = vec![];
    for entry in fs::read_dir(root).expect("Could not read the directory") {
        match entry {
            Ok(ent) => entries.push(ent),
            Err(e) => {
                eprintln!("Error reading directory entry: {:?}", &e);
            }
        }
    }
    /*
     * JSON definitions pull in all the dependent definitions
     * into a self-sufficient file, thus creating effectively duplicate
     * definitions.
     *
     * We apply heuristic of "The first seen user owns the definition",
     * which means that we must first process the files that are likely
     * to be the origin - thus "core" should go before "plugins".
     *
     * Admittedly lexicographic sort is a bit of a hack/overkill,
     * but it does that part of the job very nicely.
     */
    entries.sort_by(|a,b| a.path().cmp(&b.path()));

    for entry in &entries {
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
    if apitype.starts_with("vl_api_") {
        // let ctype_trimmed = apitype.trim_start_matches("vl_api_").trim_end_matches("_t");
        // String::from(ctype_trimmed)
        camelize_ident(apitype.trim_start_matches("vl_api_").trim_end_matches("_t"))
        // camelize_ident(ctype_trimmed)
    } else {
        if apitype == "string" {
            format!("String")
        } else {
            format!("{}", apitype)
        }
    }
}
pub fn get_ident(api_ident: &str) -> String {
    if api_ident == "type" {
        return format!("typ");
    }
    if api_ident == "match" {
        // println!("Found match");
        format!("mach")
    } else {
        format!("{}", api_ident.trim_start_matches("_"))
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

pub fn camelize_ident(ident: &str) -> String {
    let mut c = ident.split("_");
    let collection: Vec<&str> = c.collect();
    let mut finalString = String::new();

    for x in collection {
        for (i, c) in x.chars().enumerate() {
            if i == 0 {
                let c_upper: Vec<_> = c.to_uppercase().collect();
                finalString.push_str(&c_upper[0].to_string());
            } else {
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
