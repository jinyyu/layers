use std::sync::Arc;
use std::rc::Rc;
use packet::Packet;
use detector;
use libc::c_char;
use std::ptr;
use std::collections::VecDeque;
use layer::flow::TcpFlow;
use std::cell::RefCell;
use layer::dissector;

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

#[allow(dead_code)]
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
    detector: Rc<detector::Detector>,
    flow: *const c_char,
    client_id: *const c_char,
    server_id: *const c_char,
    detect_times: u8,
    proto: detector::Proto,

    pending_packets: VecDeque<Arc<Packet>>,

    client_flow: Option<TcpFlow>,
    server_flow: Option<TcpFlow>,

    dissector: Rc<RefCell<dissector::TCPDissector>>,
}


impl TCPStream {
    const MAX_DETECT_TIMES: u8 = 10;

    pub fn new(packet: Arc<Packet>, detector: Rc<detector::Detector>) -> TCPStream {
        let mut stream = TCPStream {
            finished: false,
            state: State::DetectTrying,

            detector,
            flow: ptr::null(),
            client_id: ptr::null(),
            server_id: ptr::null(),
            detect_times: 0,
            proto: detector::Proto::new(),

            client_port: packet.src_port,
            server_port: packet.dst_port,

            client: packet.src_ip,
            server: packet.dst_ip,

            pending_packets: VecDeque::with_capacity(TCPStream::MAX_DETECT_TIMES as usize),

            client_flow: None,
            server_flow: None,
            dissector: dissector::TCPDissectorAllocator::default(),
        };

        if unsafe { (*packet.tcp).flags & TCPHeader::SYN > 0 } {
            debug!("syn stream");
            stream.state = State::DetectTrying;
        }
        return stream;
    }

    pub fn handle_packet(&mut self, packet: &Arc<Packet>) {
        match self.state {
            State::DetectError => {
                debug!("unknown protocol");
            }
            State::DetectTrying => {
                self.pending_packets.push_back(packet.clone());
                if self.flow == ptr::null() {
                    unsafe {
                        self.flow = detector::new_ndpi_flow();
                        self.client_id = detector::new_ndpi_flow_id();
                        self.server_id = detector::new_ndpi_flow_id();
                    }
                }
                self.detect_protocol(packet);
            }
            State::DetectSuccess => {
                self.dispatch_packet(packet);
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

    #[inline]
    fn is_client_flow(&self, packet: &Arc<Packet>) -> bool {
        return packet.src_port == self.client_port && packet.src_ip == self.client;
    }


    fn detect_protocol(&mut self, packet: &Arc<Packet>) {
        if self.is_client_flow(packet) {
            self.proto = self.detector.detect(self.flow,
                                              packet.ipv4 as *const c_char,
                                              packet.ip_layer_len as u16,
                                              packet.timestamp,
                                              self.client_id,
                                              self.server_id);
        } else {
            self.proto = self.detector.detect(self.flow,
                                              packet.ipv4 as *const c_char,
                                              packet.ip_layer_len as u16,
                                              packet.timestamp,
                                              self.server_id,
                                              self.client_id);
        }

        if self.proto.success() {
            debug!("detect success");
            self.state = State::DetectSuccess;
            self.on_detect_success();
        } else {
            self.detect_times += 1;
            if self.detect_times > TCPStream::MAX_DETECT_TIMES {
                self.state = State::DetectError;
                self.on_detect_failed()
            }
        }
    }


    fn on_detect_success(&mut self) {
        debug!("proto name = {}", self.detector.protocol_name(&self.proto));
        self.dissector = self.detector.alloc_tcp_dissector(&self.proto, self.detector.clone(), self.flow);
        loop {
            let packet = self.pending_packets.pop_front();
            match packet {
                None => {
                    break;
                }
                Some(packet) => {
                    self.dispatch_packet(&packet);
                }
            }
        }
    }

    fn dispatch_packet(&mut self, packet: &Arc<Packet>) {
        let flow;
        let is_client = self.is_client_flow(packet);
        if is_client {
            flow = &mut self.client_flow;
        } else {
            assert_eq!(packet.src_ip, self.server);
            assert_eq!(packet.src_port, self.server_port);
            flow = &mut self.server_flow;
        }
        match *flow {
            None => {
                let mut f;
                let dissector = self.dissector.clone();

                if is_client {
                    let cb = move |data: &[u8]| {
                        dissector.borrow_mut().on_client_data(data);
                    };
                    f = TcpFlow::new(packet, Box::new(cb));
                } else {
                    let cb = move |data: &[u8]| {
                        dissector.borrow_mut().on_server_data(data);
                    };
                    f = TcpFlow::new(packet, Box::new(cb));
                }
                f.handle_packet(packet);

                *flow = Some(f);
            }
            Some(ref mut flow) => {
                flow.handle_packet(packet);
            }
        }
    }

    fn on_detect_failed(&mut self) {
        self.pending_packets.clear();
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

