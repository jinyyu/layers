use env_logger::Builder;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

#[macro_use]
extern crate log;
extern crate argparse;
extern crate env_logger;
extern crate layers;

use layers::*;

struct Main {
    dispatcher: Arc<dispatcher::Dispatcher>,
    daq: Arc<daq::DAQ>,
}

impl Main {
    fn new(conf: Arc<config::Configure>) -> Main {
        let daq = daq::init(conf.clone());
        let dispatcher = dispatcher::init(conf.clone());

        Main { dispatcher, daq }
    }

    fn setup_workspace(conf: Arc<config::Configure>) {
        let path = Path::new(&*conf.workspace);
        let exists = Path::exists(path);
        if !exists {
            let result = fs::create_dir(path);
            match result {
                Ok(_) => {
                    debug!("create dir success");
                }
                Err(err) => panic!("create workspace dir error {}", err),
            }
        }
        env::set_current_dir(path).unwrap();
        debug!("setup ok");
    }

    fn run(&self) {
        let dispatcher = self.dispatcher.clone();

        self.daq.run(&move |packet: Arc<packet::Packet>| {
            dispatcher.dispatch(packet);
        });
    }
}

fn main() {
    let mut configure = "/etc/layers/config.yaml".to_string();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("layers");
        ap.refer(&mut configure).add_option(
            &["-c", "--config"],
            argparse::Store,
            "config file path",
        );
        ap.parse_args_or_exit();
    }

    Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}:{}] {}",
                record.level(),
                record.file().unwrap(),
                record.line().unwrap(),
                record.args()
            )
        })
        .init();

    let conf = config::load(configure);
    Main::setup_workspace(conf.clone());
    layer::HTTPDissector::init();
    let app = Main::new(conf);
    app.run();
}
