use crate::config::Configure;
use crate::detector::{Detector, Proto};
use crate::layer::tcp::HTTPDissector;
use libc::c_char;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

pub trait TCPDissector {
    fn on_client_data(&mut self, data: &[u8]) -> Result<(), ()>;
    fn on_server_data(&mut self, data: &[u8]) -> Result<(), ()>;
}

pub struct DefaultDissector {}

impl DefaultDissector {
    fn new() -> Rc<RefCell<TCPDissector>> {
        Rc::new(RefCell::new(DefaultDissector {}))
    }
}

impl TCPDissector for DefaultDissector {
    fn on_client_data(&mut self, _data: &[u8]) -> Result<(), ()> {
        Err(())
    }
    fn on_server_data(&mut self, _data: &[u8]) -> Result<(), ()> {
        Err(())
    }
}

pub struct TCPDissectorAllocator {
    protocol:
        HashMap<u16, fn(detector: Rc<Detector>, flow: *const c_char) -> Rc<RefCell<TCPDissector>>>,
}

impl TCPDissectorAllocator {
    pub fn new(conf: Arc<Configure>) -> TCPDissectorAllocator {
        let mut allocator = TCPDissectorAllocator {
            protocol: HashMap::new(),
        };

        if conf.is_dissector_enable("http") {
            let cb = HTTPDissector::new;
            allocator.protocol.insert(Proto::HTTP, cb);
            allocator.protocol.insert(Proto::HTTP_ACTIVESYNC, cb);
            allocator.protocol.insert(Proto::HTTP_CONNECT, cb);
            allocator.protocol.insert(Proto::HTTP_DOWNLOAD, cb);
            allocator.protocol.insert(Proto::HTTP_PROXY, cb);
        }

        allocator
    }

    pub fn default() -> Rc<RefCell<TCPDissector>> {
        Rc::new(RefCell::new(DefaultDissector {}))
    }

    pub fn alloc_dissector(
        &self,
        proto: &Proto,
        detector: Rc<Detector>,
        flow: *const c_char,
    ) -> Rc<RefCell<TCPDissector>> {
        if let Some(cb) = self.protocol.get(&proto.app_protocol) {
            return cb(detector, flow);
        }

        if let Some(cb) = self.protocol.get(&proto.master_protocol) {
            return cb(detector, flow);
        }

        DefaultDissector::new()
    }
}
