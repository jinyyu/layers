use std::sync::Arc;
use packet::Packet;
use detector::Detector;

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
    const FIN: u8 = 0x01;
    const SYN: u8 = 0x02;
    const RST: u8 = 0x04;
    const PUSH: u8 = 0x08;
    const ACK: u8 = 0x10;
    const URG: u8 = 0x20;

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
    times: u8,
}


impl TCPStream {
    const MAX_DETECT_TIMES: u8 = 10;

    pub fn new(packet: Arc<Packet>) -> TCPStream {
        let state;
        if unsafe { (*packet.tcp).flags | TCPHeader::SYN > 0 } {
            debug!("syn stream");
            state = State::DetectTrying;
        } else {
            debug!("not syn stream");
            state = State::DetectError;
        }

        TCPStream {
            finished: false,
            state,
            times: 0,
        }
    }

    pub fn handle_packet(&mut self, packet: Arc<Packet>, detector: &Detector) {
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