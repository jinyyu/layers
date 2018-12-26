use crate::config;
use crate::detector::Detector;
use crate::layer::ip::StreamID;
use crate::layer::tcp::TCPStream;
use crate::packet::Packet;
use layer::IPProto;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

pub struct TCPTracker {
    streams: HashMap<StreamID, TCPStream>,
    detector: Rc<Detector>,
    last_cleanup: u64,
}

impl TCPTracker {
    //micro second
    const STREAM_CLEANUP_DURATION: u64 = 1000 * 1000 * 30;

    pub fn new(conf: Arc<config::Configure>) -> TCPTracker {
        TCPTracker {
            last_cleanup: 0,
            streams: HashMap::new(),
            detector: Rc::new(Detector::new(conf.clone(), IPProto::TCP)),
        }
    }

    pub fn on_packet(&mut self, packet: &Arc<Packet>) {
        let id = StreamID::new(
            packet.src_ip,
            packet.dst_ip,
            packet.src_port,
            packet.dst_port,
        );

        let remove;

        let tm = packet.timestamp;
        {
            let pkt = packet.clone();
            let detector = self.detector.clone();
            let stream = self.streams.entry(id).or_insert_with(|| {
                trace!(
                    "new tcp stream {}:{} ->{}:{}",
                    pkt.src_ip_str(),
                    pkt.src_port,
                    pkt.dst_ip_str(),
                    pkt.dst_port
                );
                let stream = TCPStream::new(pkt, detector);
                return stream;
            });

            stream.handle_packet(packet);
            remove = stream.is_finished();
        }
        if remove {
            self.streams.remove(&id);
        } else {
            self.cleanup_stream(tm);
        }
    }

    pub fn cleanup_stream(&mut self, tm: u64) {
        if self.last_cleanup + TCPTracker::STREAM_CLEANUP_DURATION > tm {
            return;
        }

        let before = self.streams.len();

        self.streams
            .retain(|_k, v| -> bool { v.last_seen() + TCPTracker::STREAM_CLEANUP_DURATION > tm });

        let after = self.streams.len();
        debug!("tcp stream cleanup {}/{}", before - after, before);
        self.last_cleanup = tm;
    }
}
