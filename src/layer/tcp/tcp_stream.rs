use crate::detector;
use crate::layer::dissector;
use crate::layer::tcp_flow::TcpFlow;
use crate::packet::Packet;
use inet;
use libc::c_char;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;

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
    pub const FIN: u8 = 0x01;
    pub const SYN: u8 = 0x02;
    pub const RST: u8 = 0x04;
    pub const PUSH: u8 = 0x08;
    pub const ACK: u8 = 0x10;
    pub const URG: u8 = 0x20;

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
    skip: Rc<Cell<bool>>,
    finished: bool,
    state: State,

    last_timestamp: u64,

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

    client_flow: Option<Box<TcpFlow>>,
    server_flow: Option<Box<TcpFlow>>,

    dissector: Rc<RefCell<dissector::TCPDissector>>,
}

impl TCPStream {
    const MAX_DETECT_TIMES: u8 = 10;

    pub fn new(packet: Arc<Packet>, detector: Rc<detector::Detector>) -> TCPStream {
        let mut stream = TCPStream {
            skip: Rc::new(Cell::new(false)),
            finished: false,
            state: State::DetectTrying,

            last_timestamp: packet.timestamp,

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

    pub fn last_seen(&self) -> u64 {
        self.last_timestamp
    }

    pub fn handle_packet(&mut self, packet: &Arc<Packet>) {
        self.last_timestamp = packet.timestamp;

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
            self.proto = self.detector.detect(
                self.flow,
                packet.ipv4 as *const c_char,
                packet.ip_layer_len as u16,
                packet.timestamp,
                self.client_id,
                self.server_id,
            );
        } else {
            self.proto = self.detector.detect(
                self.flow,
                packet.ipv4 as *const c_char,
                packet.ip_layer_len as u16,
                packet.timestamp,
                self.server_id,
                self.client_id,
            );
        }

        if self.proto.success() {
            self.on_detect_success();
        } else {
            self.detect_times += 1;
            if self.detect_times > TCPStream::MAX_DETECT_TIMES {
                self.detect_give_up();
            }
        }
    }

    fn detect_give_up(&mut self) {
        if self.state != State::DetectTrying {
            return;
        }
        self.proto = self.detector.detect_give_up(self.flow);
        if self.proto.success() {
            self.on_detect_success();
            return;
        }

        self.proto = self.detector.guess_undetected_protocol(
            self.flow,
            unsafe { inet::ntohl(self.client) },
            self.client_port,
            unsafe { inet::ntohl(self.server) },
            self.server_port,
        );
        if self.proto.success() {
            self.on_detect_success();
        } else {
            self.on_detect_failed();
        }
    }

    fn on_detect_success(&mut self) {
        self.state = State::DetectSuccess;
        debug!(
            "detect success proto name = {}",
            self.detector.protocol_name(&self.proto)
        );
        self.dissector =
            self.detector
                .alloc_tcp_dissector(&self.proto, self.detector.clone(), self.flow);
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
                    let skip = self.skip.clone();
                    let cb = move |data: &[u8]| {
                        let result = dissector.borrow_mut().on_client_data(data);
                        match result {
                            Err(_) => {
                                skip.set(true);
                            }
                            Ok(_) => {}
                        }
                    };
                    f = TcpFlow::new(packet, Box::new(cb));
                } else {
                    let skip = self.skip.clone();
                    let cb = move |data: &[u8]| {
                        let result = dissector.borrow_mut().on_server_data(data);
                        match result {
                            Err(_) => {
                                skip.set(true);
                            }
                            Ok(_) => {}
                        }
                    };
                    f = TcpFlow::new(packet, Box::new(cb));
                }
                f.process_packet(packet);

                *flow = Some(f);
            }
            Some(ref mut flow) => {
                if self.skip.get() {
                    debug!("skip data");
                } else {
                    flow.process_packet(packet);
                }
            }
        }
    }

    fn on_detect_failed(&mut self) {
        self.state = State::DetectError;
        self.pending_packets.clear();
    }
}

impl Drop for TCPStream {
    fn drop(&mut self) {
        self.detect_give_up();
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
