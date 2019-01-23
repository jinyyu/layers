use detector::Detector;
use layer::ip::StreamID;
use layer::packet::Packet;
use layer::IPProto;
use layer::TCPStream;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

pub struct TCPTracker {
    streams: HashMap<StreamID, Box<TCPStream>>,
    detector: Rc<Detector>,
    last_cleanup: u64,
}

impl TCPTracker {
    //micro second
    const STREAM_CLEANUP_DURATION: u64 = 1000 * 1000 * 30;

    pub fn new() -> TCPTracker {
        TCPTracker {
            last_cleanup: 0,
            streams: HashMap::new(),
            detector: Rc::new(Detector::new(IPProto::TCP)),
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
            let stream = TCPStream::new(packet.clone(), self.detector.clone());
            match stream {
                Some(mut stream) => {
                    stream.handle_packet(packet);
                    finished = stream.is_finished();

                    if !finished {
                        self.streams.insert(id, stream);
                    }
                }
                None => trace!("not sync stream, ignore"),
            }
        }

        if finished {
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

        self.streams.retain(|_k, stream| -> bool {
            stream.last_seen() + TCPTracker::STREAM_CLEANUP_DURATION > tm
        });

        let after = self.streams.len();
        debug!("tcp stream cleanup {}/{}", before - after, before);
        self.last_cleanup = tm;
    }
}
