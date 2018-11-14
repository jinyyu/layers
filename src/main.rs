#[macro_use]
extern crate log;
extern crate env_logger;
extern crate argparse;

use std::io::Write;
use env_logger::Builder;

fn main() {
    let mut config = "/etc/filedump/config.yaml".to_string();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("file dump");
        ap.refer(&mut config)
            .add_option(&["-c", "--config"], argparse::Store,
                        "config file path");
        ap.parse_args_or_exit();
    }

    Builder::from_default_env()
        .format(|buf, record| {
            writeln!(buf, "[{}] [{}:{}] {}", record.level(), record.file().unwrap(), record.line().unwrap(), record.args())
        }).init();

    debug!("config path = {}", config)
}
