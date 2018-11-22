use std::cell::Cell;
use std::sync::Arc;
use packet::Packet;

pub struct TCPTracker {}


impl TCPTracker {
    pub fn new() -> Cell<TCPTracker> {
        let mut tracker = TCPTracker{

        };

        return Cell::new(tracker);
    }
    pub fn on_packet(&mut self,packet: Arc<Packet> ) {

        debug!("{}:{} ->{}:{}", packet.src_ip_str(), packet.src_port, packet.dst_ip_str(), packet.dst_port);
    }
}