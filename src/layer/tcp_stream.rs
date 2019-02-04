use detector;
use inet;
use layer::packet::Packet;
use layer::stream_state;
use layer::tcp::TCPHeader;
use layer::tcp::{TCPDissector, TCPDissectorAllocator};
use layer::TcpFlow;
use libc::c_char;
use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use std::vec::Vec;

pub struct TCPStream {
    state: u32,
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

    pending_packets: Rc<RefCell<Vec<Arc<Packet>>>>,

    client_flow: Option<Box<TcpFlow>>,
    server_flow: Option<Box<TcpFlow>>,

    dissector: Rc<RefCell<TCPDissector>>,
}

impl TCPStream {
    const MAX_DETECT_TIMES: u8 = 10;

    pub fn new(packet: Arc<Packet>, detector: Rc<detector::Detector>) -> Option<Box<TCPStream>> {
        if unsafe { (*packet.tcp).flags & TCPHeader::SYN == 0 } {
            return None;
        }

        let stream = Box::new(TCPStream {
            state: stream_state::STATE_PROTOCOL_DETECTING,

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

            pending_packets: Rc::new(RefCell::new(Vec::with_capacity(
                TCPStream::MAX_DETECT_TIMES as usize,
            ))),

            client_flow: None,
            server_flow: None,
            dissector: TCPDissectorAllocator::default(),
        });

        trace!("{}", stream_state::state_to_string(stream.state));
        return Some(stream);
    }

    pub fn last_seen(&self) -> u64 {
        self.last_timestamp
    }

    pub fn handle_packet(&mut self, packet: &Arc<Packet>) {
        self.last_timestamp = packet.timestamp;

        if self.state
            & (stream_state::STATE_STREAM_SKIP
                | stream_state::STATE_STREAM_FINISHED
                | stream_state::STATE_PROTOCOL_FAILED)
            > 0
        {
            trace!("skip");
            return;
        }

        if self.state & stream_state::STATE_PROTOCOL_DETECTING > 0 {
            self.pending_packets.borrow_mut().push(packet.clone());

            if self.flow == ptr::null() {
                unsafe {
                    self.flow = detector::new_ndpi_flow();
                    self.client_id = detector::new_ndpi_flow_id();
                    self.server_id = detector::new_ndpi_flow_id();
                }
            }
            self.detect_protocol(packet);
        } else if self.state & stream_state::STATE_PROTOCOL_SUCCESS > 0 {
            self.dispatch_packet(packet);
        } else {
            unreachable!()
        }

        unsafe {
            if (*packet.tcp).flags & (TCPHeader::FIN | TCPHeader::RST) > 0 {
                self.state |= stream_state::STATE_STREAM_FINISHED;
                trace!(
                    "stream finished:{}",
                    stream_state::state_to_string(self.state)
                );
            }
        }
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.state & stream_state::STATE_STREAM_FINISHED > 0
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

    fn set_skip(&mut self) {
        self.state |= stream_state::STATE_STREAM_SKIP;
        trace!("skip {}", stream_state::state_to_string(self.state));
    }

    fn detect_give_up(&mut self) {
        if self.state & stream_state::STATE_PROTOCOL_FINISHED > 0 {
            return;
        }
        self.proto = self.detector.detect_give_up(self.flow, 1);
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
        self.state &= !stream_state::STATE_PROTOCOL_ALL;
        self.state |= stream_state::STATE_PROTOCOL_SUCCESS;
        trace!(
            "detect success {},{}",
            self.detector.protocol_name(&self.proto),
            stream_state::state_to_string(self.state)
        );
        self.dissector =
            self.detector
                .alloc_tcp_dissector(&self.proto, self.detector.clone(), self.flow);

        let packets = self.pending_packets.clone();
        for packet in packets.borrow().iter() {
            self.dispatch_packet(&packet);
        }
        self.pending_packets.borrow_mut().clear();
        self.pending_packets.borrow_mut().shrink_to_fit();
    }

    fn dispatch_packet(&mut self, packet: &Arc<Packet>) {
        let flow;
        let this = self as *const TCPStream;
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
                        if let Err(_) = dissector.borrow_mut().on_client_data(data) {
                            unsafe {
                                let this = this as *mut TCPStream;
                                &(*this).set_skip();
                            }
                        }
                    };
                    f = TcpFlow::new(packet, Box::new(cb));
                } else {
                    let cb = move |data: &[u8]| {
                        if let Err(_) = dissector.borrow_mut().on_server_data(data) {
                            unsafe {
                                let this = this as *mut TCPStream;
                                &(*this).set_skip();
                            }
                        }
                    };
                    f = TcpFlow::new(packet, Box::new(cb));
                }
                f.process_packet(packet);

                *flow = Some(f);
            }
            Some(ref mut flow) => {
                flow.process_packet(packet);
            }
        }
    }

    fn on_detect_failed(&mut self) {
        self.state &= !stream_state::STATE_PROTOCOL_ALL;
        self.state |= stream_state::STATE_PROTOCOL_FAILED;
        trace!(
            "detect failed {}",
            stream_state::state_to_string(self.state)
        );
        self.pending_packets.borrow_mut().clear();
        self.pending_packets.borrow_mut().shrink_to_fit();
    }
}

impl Drop for TCPStream {
    fn drop(&mut self) {
        self.detect_give_up();
        trace!("stream clean up");

        unsafe {
            if self.flow != ptr::null() {
                detector::free_ndpi_flow(self.flow);
                detector::free_ndpi_flow_id(self.client_id);
                detector::free_ndpi_flow_id(self.server_id);
            }
        }
    }
}
