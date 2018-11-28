use std::sync::Arc;
use std::rc::Rc;
use packet::Packet;
use layer::ip::StreamID;
use std::collections::HashMap;
use layer::tcp::TCPStream;
use std::cell::RefCell;

pub struct TCPTracker {
    streams: HashMap<StreamID, TCPStream>
}


impl TCPTracker {
    pub fn new() -> TCPTracker {
        TCPTracker {
            streams: HashMap::new(),
        }
    }

    pub fn on_packet(tracker: Rc<RefCell<TCPTracker>>, packet: Arc<Packet>) {
        let id = StreamID::new(packet.src_ip, packet.dst_ip, packet.src_port, packet.dst_port);

        let p = packet.clone();
        let t = tracker.clone();
        let tm = packet.timestamp;

        let mut stream = tracker.borrow_mut().streams.entry(id).or_insert_with(|| {
            debug!("new tcp stream {}:{} ->{}:{}", p.src_ip_str(), p.src_port, p.dst_ip_str(), p.dst_port);
            let mut stream = TCPStream::new(t, p);
            return stream;
        });

        stream.handle_packet(packet);

    }


    pub fn cleanup_stream(&mut self, tm: u64) {}
}