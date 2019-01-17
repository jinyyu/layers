use crate::inet;
use crate::layer::IPProto;
use crate::layer::{EthernetHeader, EthernetType, IPV4Header, TCPHeader, VlanHeader};
use std::mem;
use std::ptr;
use std::slice;
use std::sync::Arc;

pub struct Packet {
    pub flag: u8,
    pub timestamp: u64,
    pub data: Vec<u8>,

    // host endian
    pub src_port: u16,
    pub dst_port: u16,

    // net endian
    pub src_ip: u32,
    pub dst_ip: u32,

    pub ethernet: *const EthernetHeader,
    pub ipv4: *const IPV4Header,
    pub ip_layer_len: usize,

    pub tcp: *const TCPHeader,

    payload: *const u8,
    payload_len: usize,
}

unsafe impl Send for Packet {}

unsafe impl Sync for Packet {}

impl Packet {
    pub const BAD_PACKET: u8 = (0x01 << 0);
    pub const IPV4: u8 = (0x01 << 1);
    pub const IPV6: u8 = (0x01 << 2);
    pub const ARP: u8 = (0x01 << 3);
    pub const ICMP: u8 = (0x01 << 4);
    pub const TCP: u8 = (0x01 << 5);
    pub const UDP: u8 = (0x01 << 6);
    /* now combined detections */

    pub const TCP_OR_UDP: u8 = Packet::IPV4 | Packet::UDP;
    pub const IPV4TCP: u8 = Packet::IPV4 | Packet::TCP;
    pub const IPV6TCP: u8 = Packet::IPV6 | Packet::TCP;
    pub const IPV4UDP: u8 = Packet::IPV4 | Packet::UDP;
    pub const IPV6UDP: u8 = Packet::IPV6 | Packet::UDP;

    pub fn valid(&self) -> bool {
        return self.flag & Packet::BAD_PACKET == 0;
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

    pub fn tcp_payload(&self) -> &[u8] {
        assert!(self.flag & Packet::TCP_OR_UDP > 0);
        unsafe { slice::from_raw_parts(self.payload, self.payload_len) }
    }

    pub fn new(timestamp: u64, data: *const u8, size: usize) -> Arc<Packet> {
        let array = unsafe { slice::from_raw_parts(data, size) };

        let mut packet = Packet {
            flag: 0,
            data: Vec::from(array),
            timestamp,
            ethernet: ptr::null(),
            ipv4: ptr::null(),
            ip_layer_len: 0,
            tcp: ptr::null(),

            src_port: 0,
            dst_port: 0,
            src_ip: 0,
            dst_ip: 0,

            payload: ptr::null(),
            payload_len: 0,
        };
        if size < mem::size_of::<EthernetHeader>() {
            debug!("invalid packet, size = {}", size);
            packet.flag |= Packet::BAD_PACKET;
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
                self.flag |= Packet::IPV4;
                self.decode_ipv4(offset, left);
            }
            EthernetType::VLAN => {
                self.decode_vlan(offset, left)
            }
            EthernetType::T8021QINQ => {
                self.decode_vlan(offset, left)
            }
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
            self.flag |= Packet::BAD_PACKET;
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
                self.flag |= Packet::IPV4;
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
        assert!(self.flag & Packet::IPV4 > 0);

        if left < mem::size_of::<IPV4Header>() {
            self.flag |= Packet::BAD_PACKET;
            return;
        }

        let ipv4_header = unsafe { self.data.as_ptr().offset(offset as isize) as *mut IPV4Header };
        let ip = unsafe { &mut *ipv4_header };

        self.ipv4 = ipv4_header;
        self.src_ip = ip.src;
        self.dst_ip = ip.dst;

        if ip.version() != 4 {
            debug!("bad version {}", ip.version());
            self.flag |= Packet::BAD_PACKET;
            return;
        }
        let header_len = ip.header_len() as usize;
        let ip_layer_len = ip.total_length() as usize;
        self.ip_layer_len = ip_layer_len;

        if left < self.ip_layer_len || self.ip_layer_len < header_len {
            debug!("bad packet {}, {}, {}", left, self.ip_layer_len, header_len);
            self.flag |= Packet::BAD_PACKET;
            return;
        }

        let proto = IPProto(ip.proto);

        match proto {
            IPProto::TCP => {
                self.flag |= Packet::TCP;
                self.decode_tcp(offset + header_len, ip_layer_len - header_len);
            }

            _ => {
                trace!("ip type {}", proto.to_string());
            }
        }
    }

    fn decode_tcp(&mut self, offset: usize, left: usize) {
        assert!(self.flag & Packet::TCP > 0);
        let tcp = unsafe { &mut *(self.data.as_ptr().offset(offset as isize) as *mut TCPHeader) };

        self.tcp = tcp;
        self.src_port = unsafe { inet::ntohs(tcp.sport) };
        self.dst_port = unsafe { inet::ntohs(tcp.dport) };

        let header_len = unsafe { (*self.tcp).header_len() as usize };
        if left < header_len {
            debug!("bad tcp packet {} {}", left, header_len);
            self.flag |= Packet::BAD_PACKET;
            return;
        }

        self.payload_len = left - header_len;
        let offset = offset + header_len;
        self.payload = unsafe { self.data.as_ptr().offset(offset as isize) as *const u8 };
    }
}
