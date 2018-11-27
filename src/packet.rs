use std::slice;
use std::sync::Arc;
use layer::*;
use inet;
use std::ptr;
use std::mem;
use layer;


pub const FLAG_BAD_PACKET: u8 = (0x01 << 0);
pub const FLAG_IPV4: u8 = (0x01 << 1);
pub const FLAG_IPV6: u8 = (0x01 << 2);
pub const FLAG_ARP: u8 = (0x01 << 3);
pub const FLAG_ICMP: u8 = (0x01 << 4);
pub const FLAG_TCP: u8 = (0x01 << 5);
pub const FLAG_UDP: u8 = (0x01 << 6);

/* now combined detections */
pub const FLAG_IPV4TCP: u8 = FLAG_IPV4 | FLAG_TCP;
pub const FLAG_IPV6TCP: u8 = FLAG_IPV6 | FLAG_TCP;
pub const FLAG_IPV4UDP: u8 = FLAG_IPV4 | FLAG_UDP;
pub const FLAG_IPV6UDP: u8 = FLAG_IPV6 | FLAG_UDP;


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
}

unsafe impl Send for Packet {}

unsafe impl Sync for Packet {}


impl Packet {
    pub fn valid(&self) -> bool {
        return self.flag & FLAG_BAD_PACKET == 0;
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
        };
        if size < mem::size_of::<EthernetHeader>() {
            debug!("invalid packet, size = {}", size);
            packet.flag |= FLAG_BAD_PACKET;
        } else {
            packet.decode_ethernet();
        }
        return Arc::new(packet);
    }

    #[allow(non_snake_case)]
    fn decode_ethernet(&mut self) {
        let mut offset: usize = 0;
        let mut left: usize = self.data.len();
        self.ethernet = self.data.as_ptr() as *const EthernetHeader;

        offset += mem::size_of::<EthernetHeader>();
        left -= mem::size_of::<EthernetHeader>();
        unsafe {
            let eth_type = EthernetType(inet::ntohs((*self.ethernet).eth_type));
            match eth_type {
                EthernetType::IP => {
                    self.flag |= FLAG_IPV4;
                    self.decode_ipv4(offset, left);
                }
                _ => {
                    trace!("ethernet type {}", EthernetType::ethernet_type_string(eth_type));
                }
            }
        }
    }


    fn decode_ipv4(&mut self, offset: usize, left: usize) {
        assert!(self.flag & FLAG_IPV4 > 0);

        if left < mem::size_of::<IPV4Header>() {
            self.flag |= FLAG_BAD_PACKET;
            return;
        }

        unsafe {
            let ip = self.data.as_ptr().offset(offset as isize) as *const IPV4Header;
            self.ipv4 = ip;
            self.src_ip = (*ip).src;
            self.dst_ip = (*ip).dst;

            if (*ip).version() != 4 {
                debug!("bad version {}", (*self.ipv4).version());
                self.flag |= FLAG_BAD_PACKET;
                return;
            }
            let header_len = (*ip).header_len() as usize;

            if left < header_len {
                debug!("bad packet {}, {}", left, header_len);
                self.flag |= FLAG_BAD_PACKET;
                return;
            }

            self.ip_layer_len = left;

           let proto = layer::IPProto((*ip).proto);

            match proto {
                IPProto::TCP => {
                    self.flag |= FLAG_TCP;
                    self.decode_tcp(offset + header_len, left - header_len);
                }

                _ => {
                    trace!("ip type {}", proto.to_string());
                }
            }

        }
    }

    fn decode_tcp(&mut self, offset: usize, left: usize) {
        assert!(self.flag & FLAG_TCP > 0);
        unsafe {
            let tcp = self.data.as_ptr().offset(offset as isize) as *const TCPHeader;
            self.tcp = tcp;
            self.src_port = inet::ntohs((*tcp).sport);
            self.dst_port = inet::ntohs((*tcp).dport);
        }
    }
}
