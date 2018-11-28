use std::sync::Arc;
use std::rc::Rc;
use packet::Packet;
use layer::ip::StreamID;
use std::collections::HashMap;
use layer::tcp::TCPStream;

pub struct TCPTracker {
    streams: HashMap<StreamID, TCPStream>
}


impl TCPTracker {
    pub fn new() -> TCPTracker {
        TCPTracker {
            streams: HashMap::new(),
        }
    }

    pub fn on_packet(&mut self, packet: Arc<Packet>) {
        let id = StreamID::new(packet.src_ip, packet.dst_ip, packet.src_port, packet.dst_port);

        let mut remove = false;

        let tm = packet.timestamp;
        {
            let p = packet.clone();
            let stream = self.streams.entry(id).or_insert_with(|| {
                debug!("new tcp stream {}:{} ->{}:{}", p.src_ip_str(), p.src_port, p.dst_ip_str(), p.dst_port);

                let stream = TCPStream::new(p);
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