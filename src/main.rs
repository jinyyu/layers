#[macro_use]
extern crate log;
extern crate env_logger;

use std::io::Write;
use env_logger::Builder;


fn main() {
    Builder::from_default_env()
        .format(|buf, record| {
            writeln!(buf, "[{}] [{}:{}] {}", record.level(), record.file().unwrap(), record.line().unwrap(), record.args())
        })
        .init();

    debug!("hihihi")
}
