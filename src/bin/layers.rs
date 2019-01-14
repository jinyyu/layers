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

use layers::*;

type SignalCallback = extern "C" fn(_sig: i32);

extern "C" {
    fn signal(_sig: i32, _handler: SignalCallback) -> SignalCallback;
}

lazy_static! {
    static ref APP_PTR: AtomicPtr<Main> = AtomicPtr::new(ptr::null_mut());
}

extern "C" fn on_signal(sig: i32) {
    debug!("on signal {}", sig);
    let app = APP_PTR.load(Ordering::SeqCst);
    unsafe {
        if app != ptr::null_mut() {
            (*app).stop();
        }
    }
}

struct Main {
    _config: Box<config::Configure>,
    dispatcher: Arc<dispatcher::Dispatcher>,
    daq: Arc<daq::DAQ>,
}

impl Main {
    fn new(config: Box<config::Configure>) -> Main {
        mime::MimeParser::init();
        let daq = daq::init(&config.interface);
        let dispatcher = dispatcher::init(config.worker_thread as u8);

        Main {
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

impl Drop for Main {
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
    Main::setup_workspace(&conf.workspace);
    let app = Main::new(conf);

    let ptr = &app as *const Main as *mut Main;
    APP_PTR.store(ptr, Ordering::SeqCst);

    unsafe {
        signal(1, on_signal);
        signal(2, on_signal);
    }

    app.run();
}
