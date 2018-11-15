extern crate yaml_rust;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use self::yaml_rust::yaml;

pub struct Configure {
    interface: String,
    workspace: String,
}


fn print_indent(indent: usize) {
    for _ in 0..indent {
        print!("    ");
    }
}

fn dump_node(doc: &yaml::Yaml, indent: usize) {
    match *doc {
        yaml::Yaml::Array(ref v) => {
            for x in v {
                dump_node(x, indent + 1);
            }
        }
        yaml::Yaml::Hash(ref h) => {
            for (k, v) in h {
                print_indent(indent);
                debug!("{:?}:", k);
                dump_node(v, indent + 1);
            }
        }
        _ => {
            print_indent(indent);
            debug!("{:?}", doc);
        }
    }
}


pub fn load(path: String) -> Configure {
    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    let docs = yaml::YamlLoader::load_from_str(&s).unwrap();
    assert_eq!(docs.capacity(), 1);
    let doc = &docs[0];

    let interface = doc["interface"].as_str().unwrap();
    info!("interface = {}", interface);

    let workspace = doc["workspace"].as_str().unwrap();
    info!("workspace = {}", workspace);
    Configure {
        interface: interface.to_string(),
        workspace: workspace.to_string(),
    }
}