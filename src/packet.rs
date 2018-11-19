use std::slice;
use std::rc::Rc;
use layer::*;
use inet;
use std::ptr;
use std::mem;
use libc::{c_int, c_char};
use std::ffi::CString;


const FLAG_BAD_PACKET: u8 = (0x01 << 0);
const FLAG_IPV4: u8 = (0x01 << 1);
const FLAG_IPV6: u8 = (0x01 << 2);
const FLAG_ARP: u8 = (0x01 << 3);
const FLAG_ICMP: u8 = (0x01 << 4);
const FLAG_TCP: u8 = (0x01 << 5);
const FLAG_UDP: u8 = (0x01 << 6);

/* now combined detections */
const FLAG_IPV4TCP: u8 = FLAG_IPV4 | FLAG_TCP;
const FLAG_IPV6TCP: u8 = FLAG_IPV6 | FLAG_TCP;
const FLAG_IPV4UDP: u8 = FLAG_IPV4 | FLAG_UDP;
const FLAG_IPV6UDP: u8 = FLAG_IPV6 | FLAG_UDP;

const AF_INET: u32 = 2; //* IP protocol family


pub struct Packet {
    pub flag: u8,
    pub timestamp: u64,
    pub data: Vec<u8>,

    pub ethernet: *const EthernetHeader,
    pub ipv4: *const IPV4Header,
}


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

    #[inline]
    pub fn src_ip(&self) -> u32 {
        unsafe {
            return (*self.ipv4).src;
        }
    }

    #[inline]
    pub fn dst_ip(&self) -> u32 {
        unsafe {
            return (*self.ipv4).dst;
        }
    }

    pub fn src_ip_str(&self) -> String {
        let mut array:Vec<u8> = vec![0; 16];
        let mut ip = self.src_ip() as i32;
        unsafe {
            let p = &ip as *const i32;
            inet::inet_ntop(AF_INET as c_int, p as *const c_char, array.as_mut_ptr() as *mut c_char, 16);
            return CString::from_vec_unchecked(array).into_string().unwrap();
        }
    }

    pub fn dst_ip_str(&self) -> String {
        let mut array:Vec<u8> = vec![0; 16];
        let mut ip = self.dst_ip() as i32;
        unsafe {
            let p = &ip as *const i32;
            inet::inet_ntop(AF_INET as c_int, p as *const c_char, array.as_mut_ptr() as *mut c_char, 16);
            return CString::from_vec_unchecked(array).into_string().unwrap();
        }
    }


    pub fn new(timestamp: u64, data: *const u8, size: usize) -> Rc<Packet> {
        println!("data len = {}", size);
        let array = unsafe { slice::from_raw_parts(data, size) };

        let mut packet = Packet {
            flag: 0,
            data: Vec::from(array),
            timestamp,
            ethernet: ptr::null(),
            ipv4: ptr::null(),
        };
        if size < mem::size_of::<EthernetHeader>() {
            debug!("invalid packet, size = {}", size);
            packet.flag |= FLAG_BAD_PACKET;
        } else {
            packet.decode_ethernet();
        }
        return Rc::new(packet);
    }

    #[allow(non_snake_case)]
    fn decode_ethernet(&mut self) {
        let mut offset: usize = 0;
        let mut left: usize = self.data.len();
        self.ethernet = self.data.as_ptr() as *const EthernetHeader;
        println!("{} - > {}", self.src_mac(), self.dst_mac());

        offset += mem::size_of::<EthernetHeader>();
        left -= mem::size_of::<EthernetHeader>();
        unsafe {
            let eth_type = inet::ntohs((*self.ethernet).eth_type);
            match eth_type {
                ETHERNET_TYPE_IP => {
                    self.flag |= FLAG_IPV4;
                    offset = self.decode_ipv4(offset, left);

                    println!("{} - > {}", self.src_ip_str(), self.dst_ip_str());
                }
                _ => {
                    debug!("ethernet type {}", ethernet_type_string(eth_type));
                }
            }
        }
    }


    fn decode_ipv4(&mut self, offset: usize, left: usize) -> usize {
        assert!(self.flag & FLAG_IPV4 > 0);

        if left < mem::size_of::<IPV4Header>() {
            self.flag |= FLAG_BAD_PACKET;
            return 0;
        }

        unsafe {
            self.ipv4 = self.data.as_ptr().offset(offset as isize) as *const IPV4Header;
            if (*self.ipv4).version() != 4 {
                debug!("invalid ip version = {}", (*self.ipv4).version());
                self.flag |= FLAG_BAD_PACKET;
                return 0;
            }

            debug!("header len = {}", (*self.ipv4).header_len())
        }

        0
    }
}
