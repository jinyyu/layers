use std::slice;
use std::rc::Rc;
use layer::*;
use inet;
use std::ptr;
use std::mem;


const FLAG_BAD_PACKET: u8 = (0x01 << 0);
const FLAG_IPV4: u8 = (0x01 << 1);
const FLAG_IPV6: u8 = (0x01 << 2);
const FLAG_ARP: u8 = (0x01 << 3);
const FLAG_ICMP: u8 = (0x01 << 4);
const FLAG_TCP: u8 = (0x01 << 5);
const FLAG_UDP: u8 = (0x01 << 6);

//combines
const FLAG_IPV4TCP: u8 = FLAG_IPV4 | FLAG_TCP;
const FLAG_IPV6TCP: u8 = FLAG_IPV6 | FLAG_TCP;
const FLAG_IPV4UDP: u8 = FLAG_IPV4 | FLAG_UDP;
const FLAG_IPV6UDP: u8 = FLAG_IPV6 | FLAG_UDP;


pub struct Packet {
    pub timestamp: u64,
    pub data: Vec<u8>,

    pub ethernet: *const EthernetHeader,
    pub ipv4: *const IPV4Header,
}


impl Packet {
    pub fn src_mac(&self) -> String {
        unsafe {
            return (*self.ethernet).src_mac();
        }
    }

    pub fn dst_mac(&self) -> String {
        unsafe {
            return (*self.ethernet).dst_mac();
        }
    }

    pub fn new(timestamp: u64, data: *const u8, size: usize) -> Rc<Packet> {
        debug!("data len = {}", size);
        let array = unsafe { slice::from_raw_parts(data, size) };

        let mut packet = Packet {
            data: Vec::from(array),
            timestamp,
            ethernet: ptr::null(),
            ipv4: ptr::null(),
        };

        packet.decode_ethernet();

        return Rc::new(packet);
    }

    #[allow(non_snake_case)]
    fn decode_ethernet(&mut self) {
        let mut offset: usize = 0;
        let mut left: usize = self.data.len();
        self.ethernet = self.data.as_ptr() as *const EthernetHeader;
        debug!("{} - > {}", self.src_mac(), self.dst_mac());

        offset += mem::size_of::<EthernetHeader>();
        left -= mem::size_of::<EthernetHeader>();
        unsafe {
            let eth_type = inet::ntohs((*self.ethernet).eth_type);
            match eth_type {
                ETHERNET_TYPE_IP => {
                    offset = self.decode_ipv4(offset, left);
                }
                _ => {
                    debug!("ethernet type {}", ethernet_type_string(eth_type));
                }
            }
        }
    }


    fn decode_ipv4(&mut self, offset: usize, left: usize) -> usize {
        unsafe {
            self.ipv4 = self.data.as_ptr().offset(offset as isize) as *const IPV4Header;
            debug!("version = {}", (*self.ipv4).version());
        }

        0
    }
}
