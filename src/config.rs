use std::fs::File;
use std::io::prelude::*;
use yaml_rust::yaml;
use std::sync::Arc;
use std::collections::HashMap;

pub struct Configure {
    pub interface: String,
    pub workspace: String,
    pub worker_thread: i64,
    pub dissectors: HashMap<String, ()>,
}

impl Configure {
    pub fn is_dissector_enable(&self, name: &str) -> bool {
        match self.dissectors.get(name) {
            Some(_) => {
                true
            }
            None => {
                false
            }
        }
    }
}

pub fn load(path: String) -> Arc<Configure> {
    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    let docs = yaml::YamlLoader::load_from_str(&s).unwrap();
    assert_eq!(docs.capacity(), 1);
    let doc = &docs[0];

    let interface = doc["interface"].as_str().expect("invalid interface");
    info!("interface = {}", interface);

    let workspace = doc["workspace"].as_str().expect("invalid workspace");
    info!("workspace = {}", workspace);

    let worker_thread = doc["worker_thread"].as_i64().expect("invalid worker_thread");
    info!("worker_thread = {}", worker_thread);

    let mut conf = Configure {
        interface: interface.to_string(),
        workspace: workspace.to_string(),
        worker_thread,
        dissectors: HashMap::new(),
    };

    let dissectors = doc["dissector"].as_vec().expect("invalid dissector config");
    for dissector in dissectors.iter() {
        let dissector = dissector.as_str().expect("invalid config");
        info!("dissector {}", dissector);
        conf.dissectors.insert(dissector.to_string(), ());
    }

    return Arc::new(conf);
}