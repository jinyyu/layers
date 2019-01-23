use crate::inet;
use crate::layer::IPProto;
use crate::layer::{EthernetHeader, EthernetType, IPv4Header, TCPHeader, VlanHeader, UDPHeader};
use std::mem;
use std::ptr;
use std::slice;
use std::sync::Arc;

pub struct Packet {
    pub state: u32,
    pub timestamp: u64,
    pub data: Vec<u8>,

    // host endian
    pub src_port: u16,
    pub dst_port: u16,

    // net endian
    pub src_ip: u32,
    pub dst_ip: u32,

    pub ethernet: *const EthernetHeader,
    pub ipv4: *const IPv4Header,
    pub ip_layer_len: usize,

    pub tcp: *const TCPHeader,
    pub udp: *const UDPHeader,

    payload: *const u8,
    payload_len: usize,
}

unsafe impl Send for Packet {}

unsafe impl Sync for Packet {}

impl Packet {
    pub const STATE_NONE: u32 = 0;
    pub const BAD_PACKET: u32 = 1 << 0;
    pub const STATE_IPV4: u32 = 1 << 1;
    pub const STATE_IPV6: u32 = 1 << 2;
    pub const STATE_ARP: u32 = 1 << 3;
    pub const STATE_ICMP: u32 = 1 << 4;
    pub const STATE_TCP: u32 = 1 << 5;
    pub const STATE_UDP: u32 = 1 << 6;
    pub const STATE_PAYLOAD: u32 = 1 << 7;

    pub fn valid(&self) -> bool {
        return self.state & Packet::BAD_PACKET == 0;
    }

    #[inline]
    pub fn src_mac(&self) -> String {
        unsafe {
            return (*self.ethernet).src_mac();
        }
    }

    #[inline]
    pub fn dst_mac(&self) -> String {
        unsafe {
            return (*self.ethernet).dst_mac();
        }
    }

    pub fn src_ip_str(&self) -> String {
        return inet::ip_to_string(self.src_ip);
    }

    pub fn dst_ip_str(&self) -> String {
        return inet::ip_to_string(self.dst_ip);
    }

    pub fn payload_slice(&self) -> &[u8] {
        assert!(self.state & Packet::STATE_PAYLOAD > 0);
        unsafe { slice::from_raw_parts(self.payload, self.payload_len) }
    }

    pub fn new(timestamp: u64, data: *const u8, size: usize) -> Arc<Packet> {
        let array = unsafe { slice::from_raw_parts(data, size) };

        let mut packet = Packet {
            state: 0,
            data: Vec::from(array),
            timestamp,
            ethernet: ptr::null(),
            ipv4: ptr::null(),
            ip_layer_len: 0,
            tcp: ptr::null(),
            udp: ptr::null(),

            src_port: 0,
            dst_port: 0,
            src_ip: 0,
            dst_ip: 0,

            payload: ptr::null(),
            payload_len: 0,
        };
        if size < mem::size_of::<EthernetHeader>() {
            debug!("invalid packet, size = {}", size);
            packet.state |= Packet::BAD_PACKET;
        } else {
            packet.decode_ethernet();
        }
        return Arc::new(packet);
    }

    fn decode_ethernet(&mut self) {
        let mut offset: usize = 0;
        let mut left: usize = self.data.len();
        self.ethernet = self.data.as_ptr() as *const EthernetHeader;

        offset += mem::size_of::<EthernetHeader>();
        left -= mem::size_of::<EthernetHeader>();

        let eth_type = unsafe { EthernetType(inet::ntohs((*self.ethernet).eth_type)) };

        match eth_type {
            EthernetType::IP => {
                self.state |= Packet::STATE_IPV4;
                self.decode_ipv4(offset, left);
            }
            EthernetType::VLAN => self.decode_vlan(offset, left),
            EthernetType::T8021QINQ => self.decode_vlan(offset, left),
            _ => {
                trace!(
                    "ethernet type {}",
                    EthernetType::ethernet_type_string(eth_type)
                );
            }
        }
    }

    fn decode_vlan(&mut self, offset: usize, left: usize) {
        if left < mem::size_of::<VlanHeader>() {
            self.state |= Packet::BAD_PACKET;
            return;
        }
        let eth_type;
        unsafe {
            let vlan = self.data.as_ptr().offset(offset as isize) as *const VlanHeader;
            let vlan = &*vlan;
            eth_type = EthernetType(inet::ntohs(vlan.eth_type));
        }

        let offset = offset + mem::size_of::<VlanHeader>();
        let left = left - mem::size_of::<VlanHeader>();

        match eth_type {
            EthernetType::IP => {
                self.state |= Packet::STATE_IPV4;
                self.decode_ipv4(offset, left);
            }
            _ => {
                trace!(
                    "ethernet type {}",
                    EthernetType::ethernet_type_string(eth_type)
                );
            }
        }
    }

    fn decode_ipv4(&mut self, offset: usize, left: usize) {
        assert!(self.state & Packet::STATE_IPV4 > 0);

        if left < mem::size_of::<IPv4Header>() {
            self.state |= Packet::BAD_PACKET;
            return;
        }

        let ipv4_header = unsafe { self.data.as_ptr().offset(offset as isize) as *mut IPv4Header };
        let ip = unsafe { &mut *ipv4_header };

        self.ipv4 = ipv4_header;
        self.src_ip = ip.src;
        self.dst_ip = ip.dst;

        if ip.version() != 4 {
            debug!("bad version {}", ip.version());
            self.state |= Packet::BAD_PACKET;
            return;
        }
        let header_len = ip.header_len() as usize;
        let ip_layer_len = ip.total_length() as usize;
        self.ip_layer_len = ip_layer_len;

        if left < self.ip_layer_len || self.ip_layer_len < header_len {
            debug!("bad packet {}, {}, {}", left, self.ip_layer_len, header_len);
            self.state |= Packet::BAD_PACKET;
            return;
        }

        let proto = IPProto(ip.proto);

        match proto {
            IPProto::TCP => {
                self.state |= Packet::STATE_TCP;
                self.decode_tcp(offset + header_len, ip_layer_len - header_len);
            }
            IPProto::UDP => {
                self.state |= Packet::STATE_UDP;
                self.decode_udp(offset + header_len, ip_layer_len - header_len);
            }

            _ => {
                trace!("ip type {}", proto.to_string());
            }
        }
    }

    fn decode_tcp(&mut self, offset: usize, left: usize) {
        assert!(self.state & Packet::STATE_TCP > 0);
        let tcp = unsafe { &mut *(self.data.as_ptr().offset(offset as isize) as *mut TCPHeader) };

        self.tcp = tcp;
        self.src_port = unsafe { inet::ntohs(tcp.src_port) };
        self.dst_port = unsafe { inet::ntohs(tcp.dst_port) };

        let header_len = unsafe { (*self.tcp).header_len() as usize };
        if left < header_len {
            debug!("bad tcp packet {} {}", left, header_len);
            self.state |= Packet::BAD_PACKET;
            return;
        }

        self.payload_len = left - header_len;
        if self.payload_len > 0 {
            self.state |= Packet::STATE_PAYLOAD;
            let offset = offset + header_len;
            self.payload = unsafe { self.data.as_ptr().offset(offset as isize) as *const u8 };
        }
    }

    fn decode_udp(&mut self, offset: usize, left: usize) {
        assert!(self.state & Packet::STATE_UDP > 0);
        let udp = unsafe { &mut *(self.data.as_ptr().offset(offset as isize) as *mut UDPHeader) };

        self.udp = udp;
        self.src_port = unsafe { inet::ntohs(udp.src_port) };
        self.dst_port = unsafe { inet::ntohs(udp.dst_port) };

        let total_len = unsafe { inet::htons(udp.len) as usize };
        if left < total_len  {
            debug!("bad tcp packet {} {}", left, total_len);
            self.state |= Packet::BAD_PACKET;
            return;
        }

        self.payload_len = left - mem::size_of::<UDPHeader>();
        if self.payload_len > 0 {
            self.state |= Packet::STATE_PAYLOAD;
            let offset = offset + mem::size_of::<UDPHeader>();
            self.payload = unsafe { self.data.as_ptr().offset(offset as isize) as *const u8 };
        }
    }
}
