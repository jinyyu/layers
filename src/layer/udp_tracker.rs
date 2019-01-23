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

    //micro second
    const STREAM_CLEANUP_DURATION: u64 = 1000 * 1000 * 30;

    pub fn new() -> UDPTracker {
        UDPTracker {
            streams: HashMap::new(),
            detector: Rc::new(Detector::new(IPProto::UDP)),
            last_cleanup: 0,
        }
    }

    pub fn on_packet(&mut self, packet: &Arc<Packet>) {
        let id = StreamID::new(
            packet.src_ip,
            packet.dst_ip,
            packet.src_port,
            packet.dst_port,
        );
        let mut finished = false;
        let mut find = false;
        let tm = packet.timestamp;

        {
            let mut result = self.streams.get_mut(&id);

            match result {
                Some(ref mut stream) => {
                    find = true;
                    stream.handle_packet(packet);
                    finished = stream.is_finished();
                }
                None => {}
            }
        }

        if !find {
            let mut  stream = UDPStream::new(packet.clone(), self.detector.clone());
            stream.handle_packet(packet);
            finished = stream.is_finished();

            if !finished {
                self.streams.insert(id, stream);
            }
        }

        if finished {
            self.streams.remove(&id);
        } else {
            self.cleanup_stream(tm);
        }
    }

    pub fn cleanup_stream(&mut self, tm: u64) {
        if self.last_cleanup + UDPTracker::STREAM_CLEANUP_DURATION > tm {
            return;
        }

        let before = self.streams.len();

        self.streams.retain(|_k, stream| -> bool {
            stream.last_seen() + UDPTracker::STREAM_CLEANUP_DURATION > tm
        });

        let after = self.streams.len();
        debug!("udp stream cleanup {}/{}", before - after, before);
        self.last_cleanup = tm;
    }
}
