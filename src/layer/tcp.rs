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


impl TCPHeader {
    pub fn header_len(&self) -> u8 {
        return (self.off & 0xf0) >> 2;
    }
}

enum State {
    DetectTrying,
    DetectSuccess,
    DetectError,
}

pub struct TCPStream {
    finished: bool,
    state: State,
}


impl TCPStream {
    pub fn new(packet: Arc<Packet>) -> TCPStream {
        TCPStream {
            finished: false,
            state: State::DetectError,
        }
    }

    pub fn handle_packet(&mut self, packet: Arc<Packet>) {
        debug!("{}:{} ->{}:{}", packet.src_ip_str(), packet.src_port, packet.dst_ip_str(), packet.dst_port);

        let payload = packet.tcp_payload();
        unsafe {
            debug!("len = {} {}", payload.len(), (*packet.tcp).header_len());
        }

    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}