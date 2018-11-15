#[macro_use]
extern crate log;
extern crate env_logger;
extern crate argparse;

use std::io::Write;
use env_logger::Builder;

mod config;

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

    config::load(configure);
}
