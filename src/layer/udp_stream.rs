use detector;
use inet;
use layer::packet::Packet;
use layer::stream_state;
use layer::udp::dissector::UDPDissector;
use layer::udp::DefaultDissector;
use libc::c_char;
use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use std::vec::Vec;

pub struct UDPStream {
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

    dissector: Rc<RefCell<UDPDissector>>,
}

impl UDPStream {
    const MAX_DETECT_TIMES: u8 = 10;

    pub fn new(packet: Arc<Packet>, detector: Rc<detector::Detector>) -> Box<UDPStream> {
        Box::new(UDPStream {
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
                UDPStream::MAX_DETECT_TIMES as usize,
            ))),

            dissector: DefaultDissector::default(),
        })
    }
}

impl UDPStream {
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
            debug!("dispatch");
            self.dispatch_packet(packet);
        } else {
            unreachable!()
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

    pub fn last_seen(&self) -> u64 {
        self.last_timestamp
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
            if self.detect_times > UDPStream::MAX_DETECT_TIMES {
                self.detect_give_up();
            }
        }
    }

    fn dispatch_packet(&mut self, packet: &Arc<Packet>) {
        let is_client = self.is_client_flow(packet);
        let result;
        if is_client {
            result = self.dissector.borrow_mut().on_client_packet(packet);
        } else {
            debug!("------------server");
            result = self.dissector.borrow_mut().on_server_packet(packet);
        }
        match result {
            Ok(_) => {
                debug!("ok");
            }
            Err(_) => {
                debug!("set skip");
                self.set_skip();
            }
        }
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
                .alloc_udp_dissector(&self.proto, self.detector.clone(), self.flow);

        let packets = self.pending_packets.clone();
        for packet in packets.borrow().iter() {
            self.dispatch_packet(&packet);
        }
        self.pending_packets.borrow_mut().clear();
        self.pending_packets.borrow_mut().shrink_to_fit();
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

    fn set_skip(&mut self) {
        self.state |= stream_state::STATE_STREAM_SKIP;
        trace!("skip {}", stream_state::state_to_string(self.state));
    }
}
