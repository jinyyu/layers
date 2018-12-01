use std::cell::RefCell;
use std::rc::Rc;
use config::Configure;
use std::sync::Arc;
use detector::Proto;


pub trait TCPDissector {
    fn on_client_data(&mut self, data: &[u8]);
    fn on_server_data(&mut self, data: &[u8]);
}


pub struct DefaultDissector {}


impl TCPDissector for DefaultDissector {
    fn on_client_data(&mut self, data: &[u8]) {
        debug!("on client data {}", data.len());
    }
    fn on_server_data(&mut self, data: &[u8]) {
        debug!("on server data {}", data.len());
    }
}

pub struct TCPDissectorAllocator {}

impl TCPDissectorAllocator {
    pub fn new(conf: Arc<Configure>) -> TCPDissectorAllocator {
        TCPDissectorAllocator {

        }
    }

    pub fn default() -> Rc<RefCell<TCPDissector>> {
        Rc::new(RefCell::new(DefaultDissector {}))
    }

    pub fn alloc_dissector(&self, proto: &Proto) -> Rc<RefCell<TCPDissector>> {
        Rc::new(RefCell::new(DefaultDissector {}))
    }
}