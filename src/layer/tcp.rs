use std::sync::Arc;
use packet::Packet;
use std::rc::Rc;
use layer::TCPTracker;
use std::cell::RefCell;



#[repr(C)]
pub struct TCPHeader {
    //source port
    pub sport: u16,

    // destination port
    pub dport: u16,

    // sequence number
    pub seq: u32,

    // acknowledgement number
    pub ack: u32,

    // offset and reserved
    pub off: u8,

    //flags
    pub flags: u8,

    //pkt window
    pub win: u16,

    //checksum
    pub checksum: u16,

    //urgent pointer
    pub urp: u16,
}

#[derive(Clone)]
pub struct TCPStream {
    owner: Rc<RefCell<TCPTracker>>,
}


impl TCPStream {
    pub fn new(tracker: Rc<RefCell<TCPTracker>>, packet: Arc<Packet>) -> TCPStream {
        TCPStream {
            owner: tracker,
        }
    }

    pub fn handle_packet(&mut self,  packet: Arc<Packet>) {
        debug!("------------------------{}:{} ->{}:{}", packet.src_ip_str(), packet.src_port, packet.dst_ip_str(), packet.dst_port);
    }


}