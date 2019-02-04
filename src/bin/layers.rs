use env_logger::Builder;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
#[macro_use]
extern crate log;
extern crate argparse;
extern crate env_logger;
extern crate layers;
#[macro_use]
extern crate lazy_static;

use layers::layer::*;
use layers::*;

type SignalCallback = extern "C" fn(_sig: i32);

extern "C" {
    fn signal(_sig: i32, _handler: SignalCallback) -> SignalCallback;
}

lazy_static! {
    static ref LAYER_PTR: AtomicPtr<Layers> = AtomicPtr::new(ptr::null_mut());
}

extern "C" fn on_signal(sig: i32) {
    debug!("on signal {}", sig);
    let main = LAYER_PTR.load(Ordering::SeqCst);
    unsafe {
        if main != ptr::null_mut() {
            (*main).stop();
        }
    }
}

struct Layers {
    _config: Box<config::Configure>,
    dispatcher: Arc<dispatcher::Dispatcher>,
    daq: Arc<daq::DAQ>,
}

impl Layers {
    fn new(config: Box<config::Configure>) -> Layers {
        mime::MimeParser::init();
        let daq = daq::init(&config.interface);
        let dispatcher = dispatcher::init(config.worker_thread as u8);

        Layers {
            _config: config,
            dispatcher,
            daq,
        }
    }

    fn setup_workspace(path: &str) {
        let path = Path::new(path);
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
    }

    fn run(&self) {
        let dispatcher = self.dispatcher.clone();

        self.daq.run(&move |packet: Arc<packet::Packet>| {
            dispatcher.dispatch(packet);
        });
    }

    fn stop(&mut self) {
        self.daq.stop();
        self.dispatcher.stop();
    }
}

impl Drop for Layers {
    fn drop(&mut self) {
        mime::MimeParser::shutdown();
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
    Layers::setup_workspace(&conf.workspace);
    let layer = Layers::new(conf);

    let ptr = &layer as *const Layers as *mut Layers;
    LAYER_PTR.store(ptr, Ordering::SeqCst);

    unsafe {
        signal(1, on_signal);
        signal(2, on_signal);
    }

    layer.run();
}
