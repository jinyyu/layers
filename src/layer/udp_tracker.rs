use detector::Detector;
use layer::ip::StreamID;
use layer::udp_stream::UDPStream;
use std::collections::HashMap;
use std::rc::Rc;

pub struct UDPTracker {
    streams: HashMap<StreamID, Box<UDPStream>>,
    detector: Rc<Detector>,
    last_cleanup: u64,
}
