use detector::Detector;
use layer::ip::IPProto;
use layer::ip::StreamID;
use layer::packet::Packet;
use layer::udp_stream::UDPStream;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

pub struct UDPTracker {
    streams: HashMap<StreamID, Box<UDPStream>>,
    detector: Rc<Detector>,
    last_cleanup: u64,
}

impl UDPTracker {
    const STREAM_CLEANUP_DURATION: u64 = 1000 * 1000 * 30;

    pub fn new() -> UDPTracker {
        UDPTracker {
            streams: HashMap::new(),
            detector: Rc::new(Detector::new(IPProto::UDP)),
            last_cleanup: 0,
        }
    }

    pub fn on_packet(&mut self, packet: &Arc<Packet>) {}
}
