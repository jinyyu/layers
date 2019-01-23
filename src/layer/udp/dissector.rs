use layer::packet::Packet;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::collections::HashMap;
use config::Configure;
use crate::detector::{Detector, Proto};
use libc::c_char;

pub trait UDPDissector {
    fn on_client_packet(&mut self, packet: &Arc<Packet>) -> Result<(), ()>;
    fn on_server_packet(&mut self, packet: &Arc<Packet>) -> Result<(), ()>;
}

pub struct DefaultDissector {}

impl DefaultDissector {
    fn new() -> Rc<RefCell<UDPDissector>> {
        Rc::new(RefCell::new(DefaultDissector {}))
    }

    pub fn default() -> Rc<RefCell<UDPDissector>> {
        Rc::new(RefCell::new(DefaultDissector {}))
    }
}

type DissectorAllocateCallback = Fn(Rc<Detector>, *const c_char) -> Rc<RefCell<UDPDissector>>;

pub struct UDPDissectorAllocator {
    protocol: HashMap<u16, Arc<DissectorAllocateCallback>>,
}

impl UDPDissectorAllocator {
    pub fn new() -> UDPDissectorAllocator {
        let mut allocator = UDPDissectorAllocator {
            protocol: HashMap::new(),
        };

        let conf = Configure::singleton();

        if conf.is_dissector_enable("dns") {

        }

        allocator
    }

    pub fn default() -> Rc<RefCell<UDPDissector>> {
        Rc::new(RefCell::new(DefaultDissector {}))
    }

    pub fn alloc_dissector(
        &self,
        proto: &Proto,
        detector: Rc<Detector>,
        flow: *const c_char,
    ) -> Rc<RefCell<UDPDissector>> {
        if let Some(cb) = self.protocol.get(&proto.app_protocol) {
            return cb(detector, flow);
        }

        if let Some(cb) = self.protocol.get(&proto.master_protocol) {
            return cb(detector, flow);
        }

        DefaultDissector::new()
    }
}


impl UDPDissector for DefaultDissector {
    fn on_client_packet(&mut self, _packet: &Arc<Packet>) -> Result<(), ()> {
        Ok(())
    }
    fn on_server_packet(&mut self, _packet: &Arc<Packet>) -> Result<(), ()> {
        Ok(())
    }
}
