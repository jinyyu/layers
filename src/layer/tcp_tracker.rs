use std::cell::Cell;
use std::sync::Arc;
use packet::Packet;
use layer::ip::StreamID;
use std::collections::HashMap;
use layer::tcp::TCPStream;
use std::collections::btree_map::Entry::{Occupied, Vacant};


pub struct TCPTracker {
    streams: HashMap<StreamID, Box<TCPStream>>
}


impl TCPTracker {
    pub fn new() -> Cell<TCPTracker> {
        let mut tracker = TCPTracker {
            streams: HashMap::new(),
        };

        return Cell::new(tracker);
    }

    pub fn on_packet(&mut self, packet: Arc<Packet>) {
        let id = StreamID::new(packet.src_ip, packet.dst_ip, packet.src_port, packet.dst_port);

        let p = packet.clone();


        /*


        {
            self.streams.entry(id).or_insert_with(|| {
                debug!("new ------------------------{}:{} ->{}:{}", p.src_ip_str(), p.src_port, p.dst_ip_str(), p.dst_port);

                let stream = TCPStream {};
                return Box::new(stream);
            });
        }
        */

        {
            self.streams.contains_key(&id);

            debug!("========================================================================{}", self.streams.contains_key(&id));
        }


        //stream.handle_packet(packet);
    }
}