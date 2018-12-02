use std::cell::RefCell;
use std::rc::Rc;
use config::Configure;
use std::sync::Arc;
use detector::Proto;
use std::collections::HashMap;
use layer::tcp::HTTPDissector;
use libc::c_char;
use detector::Detector;


pub trait TCPDissector {
    fn on_client_data(&mut self, data: &[u8]);
    fn on_server_data(&mut self, data: &[u8]);
}


pub struct DefaultDissector {}


impl DefaultDissector {
    fn new() -> Rc<RefCell<TCPDissector>> {
        Rc::new(RefCell::new(DefaultDissector {}))
    }
}


impl TCPDissector for DefaultDissector {
    fn on_client_data(&mut self, data: &[u8]) {
        debug!("on client data {}", data.len());
    }
    fn on_server_data(&mut self, data: &[u8]) {
        debug!("on server data {}", data.len());
    }
}

pub struct TCPDissectorAllocator {
    protocol: HashMap<u16, fn(detector: Rc<Detector>, flow: *const c_char) -> Rc<RefCell<TCPDissector>>>,
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

    pub fn alloc_dissector(&self, proto: &Proto, detector: Rc<Detector>, flow: *const c_char) -> Rc<RefCell<TCPDissector>> {
        if let Some(cb) =  self.protocol.get(&proto.app_protocol) {
            return cb(detector, flow);
        }

        if let Some(cb) =  self.protocol.get(&proto.master_protocol) {
            return cb(detector, flow);
        }

        DefaultDissector::new()
    }
}