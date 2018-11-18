use std::slice;
use std::rc::Rc;
use layer::*;
use inet;
use std::ptr;
use std::mem;

pub struct Packet {
    pub timestamp: u64,
    pub data: Vec<u8>,

    pub ethernet: *const EthernetHeader,
}


impl Packet {
    pub fn src_mac(&self)->String {
        unsafe {
            return (*self.ethernet).src_mac();
        }
    }

    pub fn dst_mac(&self)->String {
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

        unsafe {
            let eth_type = inet::ntohs((*self.ethernet).eth_type);
            match eth_type {
                ETHERNET_TYPE_IP => {
                    offset += mem::size_of::<EthernetHeader>();
                    left -= mem::size_of::<EthernetHeader>();

                    offset = self.decode_ipv4(offset, left);
                }
                _ => {
                    debug!("ethernet type {}", ethernet_type_string(eth_type));
                }
            }
        }
    }


    fn decode_ipv4(&mut self, offset: usize, left: usize) -> usize {
        0
    }
}
