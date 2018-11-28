use std::sync::Arc;
use packet::Packet;
use detector;
use libc::c_char;
use std::ptr;


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

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    DetectTrying,
    DetectSuccess,
    DetectError,
}

pub struct TCPStream {
    finished: bool,
    state: State,

    //host order
    client_port: u16,
    server_port: u16,

    //net order
    client: u32,
    server: u32,

    //ndpi
    flow: *const c_char,
    client_id: *const c_char,
    server_id: *const c_char,
    detect_times: u8,
    proto: detector::Proto,
}


impl TCPStream {
    const MAX_DETECT_TIMES: u8 = 3;

    pub fn new(packet: Arc<Packet>) -> TCPStream {
        let mut stream = TCPStream {
            finished: false,
            state: State::DetectTrying,

            flow: ptr::null(),
            client_id: ptr::null(),
            server_id: ptr::null(),
            detect_times: 0,
            proto: detector::Proto::new(),

            client_port: packet.src_port,
            server_port: packet.dst_port,

            client: packet.src_ip,
            server: packet.dst_ip,
        };

        if unsafe { (*packet.tcp).flags & TCPHeader::SYN > 0 } {
            debug!("syn stream");
            stream.state = State::DetectTrying;
        }
        return stream;
    }

    pub fn handle_packet(&mut self, packet: Arc<Packet>, detector: &detector::Detector) {
        debug!("{}:{} ->{}:{}", packet.src_ip_str(), packet.src_port, packet.dst_ip_str(), packet.dst_port);

        let payload = packet.tcp_payload();
        unsafe {
            debug!("len = {} {}", payload.len(), (*packet.tcp).header_len());
        }

        match self.state {
            State::DetectError => {
                debug!("----------------------------------------------------------unknown protocol");
            }
            State::DetectTrying => {
                debug!("---------------------------------------------------!!!!!!!!!!!!!!!!--detect");
                if self.flow == ptr::null() {
                    unsafe {
                        self.flow = detector::new_ndpi_flow();
                        self.client_id = detector::new_ndpi_flow_id();
                        self.server_id = detector::new_ndpi_flow_id();
                    }
                }
                self.detect_protocol(packet.clone(), detector);
            }
            State::DetectSuccess => {
                debug!("-----------------------------------------------------detect success");
            }
        }

        unsafe {
            if (*packet.tcp).flags & (TCPHeader::FIN | TCPHeader::RST) > 0 {
                debug!("finished");
                self.finished = true
            }
        }
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.finished
    }


    fn detect_protocol(&mut self, packet: Arc<Packet>, detector: &detector::Detector) {
        let client_flow = (packet.src_ip == self.client);
        if client_flow {
            self.proto = detector.detect(self.flow,
                                         packet.ipv4 as *const c_char,
                                         packet.ip_layer_len as u16,
                                         packet.timestamp,
                                         self.client_id,
                                         self.server_id);
        } else {
            self.proto = detector.detect(self.flow,
                                         packet.ipv4 as *const c_char,
                                         packet.ip_layer_len as u16,
                                         packet.timestamp,
                                         self.server_id,
                                         self.client_id);
        }

        if self.proto.success() {
            debug!("yes                                           detect success");
            self.state = State::DetectSuccess;
        } else {
            self.detect_times += 1;

            if self.detect_times > TCPStream::MAX_DETECT_TIMES {
                panic!("detect error ")
            }
        }


    }
}

impl Drop for TCPStream {
    fn drop(&mut self) {
        debug!("stream clean up");
        unsafe {
            if self.flow != ptr::null() {
                detector::free_ndpi_flow(self.flow);
                detector::free_ndpi_flow_id(self.client_id);
                detector::free_ndpi_flow_id(self.server_id);
            }
        }
    }
}