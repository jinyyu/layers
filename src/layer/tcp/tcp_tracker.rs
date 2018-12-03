use std::sync::Arc;
use std::rc::Rc;
use packet::Packet;
use layer::ip::StreamID;
use std::collections::HashMap;
use layer::tcp::TCPStream;
use detector::Detector;
use config;

pub struct TCPTracker {
    streams: HashMap<StreamID, TCPStream>,
    detector: Rc<Detector>,
    last_cleanup: u64,
}


impl TCPTracker {
    //micro secode
    const STREAM_CLEANUP_DURATION: u64 = 1000 * 1000 * 30;

    pub fn new(conf: Arc<config::Configure>) -> TCPTracker {
        TCPTracker {
            last_cleanup: 0,
            streams: HashMap::new(),
            detector: Rc::new(Detector::new(conf.clone())),
        }
    }

    pub fn on_packet(&mut self, packet: &Arc<Packet>) {
        let id = StreamID::new(packet.src_ip, packet.dst_ip, packet.src_port, packet.dst_port);

        let remove;

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


    pub fn cleanup_stream(&mut self, tm: u64) {
        //fixme: effective
        if self.last_cleanup + TCPTracker::STREAM_CLEANUP_DURATION > tm {
            return;
        }

        let mut keys = Vec::new();

        {
            let iter = self.streams.iter().filter(|item| {
                if item.1.last_seen() + TCPTracker::STREAM_CLEANUP_DURATION < tm {
                    return true;
                } else {
                    return false;
                }
            });

            for k in iter {
                keys.push(*k.0)
            }
        }
        let mut n_stream = 0;
        for key in keys.iter() {
            self.streams.remove(key);
            n_stream += 1
        }
        debug!("clean tcp stream {}/{}", n_stream, n_stream + self.streams.len());
        self.last_cleanup = tm;
    }
}