extern crate yaml_rust;

use std::fs::File;
use std::io::prelude::*;
use self::yaml_rust::yaml;

pub struct Configure {
    pub interface: String,
    pub workspace: String,
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