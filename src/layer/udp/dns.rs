use layer::udp::UDPDissector;
use std::sync::Arc;
use layer::packet::Packet;
use std::rc::Rc;
use crate::detector::Detector;
use libc::c_char;
use std::cell::RefCell;

pub struct DNSDissector {}

impl DNSDissector {
    pub fn new(detector: Rc<Detector>, flow: *const c_char) ->Rc<RefCell<UDPDissector>> {
        debug!("----------new dns ");
        Rc::new(RefCell::new(DNSDissector {}))
    }
}


impl UDPDissector for DNSDissector {
    fn on_client_packet(&mut self, packet: &Arc<Packet>) -> Result<(), ()> {
        debug!("client dns packet");
        Ok(())
    }
    fn on_server_packet(&mut self, packet: &Arc<Packet>) -> Result<(), ()> {
        debug!("server server packet");
        Err(())
    }
}