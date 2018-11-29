use std::sync::Arc;
use std::rc::Rc;
use packet::Packet;
use layer::ip::StreamID;
use std::collections::HashMap;
use layer::tcp::TCPStream;
use detector::Detector;

pub struct TCPTracker {
    streams: HashMap<StreamID, TCPStream>,
    detector: Rc<Detector>,
}


impl TCPTracker {
    pub fn new() -> TCPTracker {
        TCPTracker {
            streams: HashMap::new(),
            detector: Rc::new(Detector::new()),
        }
    }

    pub fn on_packet(&mut self, packet: &Arc<Packet>) {
        let id = StreamID::new(packet.src_ip, packet.dst_ip, packet.src_port, packet.dst_port);

        let mut remove = false;

        let tm = packet.timestamp;
        {
            let pkt = packet.clone();
            let detector = self.detector.clone();
            let stream = self.streams.entry(id).or_insert_with(|| {
                debug!("new tcp stream {}:{} ->{}:{}", pkt.src_ip_str(), pkt.src_port, pkt.dst_ip_str(), pkt.dst_port);
                let stream = TCPStream::new(pkt, detector);
                return stream;
            });

            stream.handle_packet(packet);
            remove = stream.is_finished();
        }
        if remove {
            self.streams.remove(&id);
        }

        self.cleanup_stream(tm);
    }


    pub fn cleanup_stream(&mut self, tm: u64) {}
}