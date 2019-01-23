use detector;
use layer::packet::Packet;
use layer::udp::dissector::UDPDissector;
use libc::c_char;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::vec::Vec;

pub struct UDPStream {
    state: u32,
    last_timestamp: u64,

    //host order
    client_port: u16,
    server_port: u16,

    //net order
    client: u32,
    server: u32,

    //ndpi
    detector: Rc<detector::Detector>,
    flow: *const c_char,
    client_id: *const c_char,
    server_id: *const c_char,
    detect_times: u8,
    proto: detector::Proto,

    pending_packets: Vec<Arc<Packet>>,

    dissector: Rc<RefCell<UDPDissector>>,
}
