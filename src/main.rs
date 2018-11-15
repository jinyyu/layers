#[macro_use]
extern crate log;
extern crate env_logger;
extern crate argparse;

use std::io::Write;
use env_logger::Builder;
use std::env;
use std::path::Path;
use std::fs;

mod config;
mod pcap;

struct Main {
    config: config::Configure,
}

impl Main {
    pub fn setup(&self){
        let path = Path::new(&self.config.workspace);
        let exists =  Path::exists(path);
        if !exists {
            let result = fs::create_dir(path);
            match result {
                Ok(_) => {
                    debug!("create dir success");
                }
                Err(err) =>{
                    panic!("create workspace dir error {}", err)
                }
            }
        }
        env::set_current_dir(path).unwrap()
    }

    pub fn run(&self) {}

}

fn main() {
    let mut configure = "/etc/layers/config.yaml".to_string();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("layers");
        ap.refer(&mut configure)
            .add_option(&["-c", "--config"], argparse::Store,
                        "config file path");
        ap.parse_args_or_exit();
    }

    Builder::from_default_env()
        .format(|buf, record| {
            writeln!(buf, "[{}] [{}:{}] {}", record.level(), record.file().unwrap(), record.line().unwrap(), record.args())
        }).init();

    let mut app = Main {
        config: config::load(configure),
    };

    app.setup();
    app.run();
}