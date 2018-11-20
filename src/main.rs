#[macro_use]
extern crate log;
extern crate env_logger;
extern crate argparse;
extern crate libc;
extern crate yaml_rust;

use std::io::Write;
use env_logger::Builder;
use std::env;
use std::path::Path;
use std::fs;
use std::rc::Rc;

mod config;
mod daq;
mod packet;
mod layer;
mod inet;
mod dispatcher;

struct Main {
    config: config::Configure,
    dispatcher: Option<Rc<dispatcher::Dispatcher>>,
    daq: Option<Box<daq::DAQ>>,
}

impl Main {
    pub fn setup(&mut self) {
        self.setup_workspace();

        self.dispatcher = Option::Some(dispatcher::init(&self.config));

        self.daq = daq::init(&self.config);
    }

    fn setup_workspace(&mut self) {
        let path = Path::new(&self.config.workspace);
        let exists = Path::exists(path);
        if !exists {
            let result = fs::create_dir(path);
            match result {
                Ok(_) => {
                    debug!("create dir success");
                }
                Err(err) => {
                    panic!("create workspace dir error {}", err)
                }
            }
        }
        env::set_current_dir(path).unwrap();
        debug!("set up ok");
    }


    pub fn run(&mut self) {
        match &self.daq {
            None => {
                panic!("inint pcap error")
            }
            Some(daq) => {
                daq.run( |packet| {
                    if packet.flag & packet::FLAG_IPV4 > 0 {
                        debug!("{}->{}", packet.src_ip_str(), packet.dst_ip_str());
                    }
                });
            }
        }
    }
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


    let conf = config::load(configure);
    let mut app = Main {
        config: conf,
        dispatcher: Option::None,
        daq: Option::None,
    };

    app.setup();

    let app = Box::new(app);

    (*app).run();
}