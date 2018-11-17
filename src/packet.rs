use std::slice;
use std::rc::Rc;
use layer::*;
use std::ptr;
use std::mem;

pub struct Packet {
    pub timestamp: u64,
    pub data: Vec<u8>,

    pub ethernet: *const EthernetHdr,
}


impl Packet {
    pub fn new(timestamp: u64, data: *const u8, size: usize) -> Rc<Packet> {
        debug!("data len = {}", size);
        let array = unsafe { slice::from_raw_parts(data, size) };

        let mut packet = Packet {
            data: Vec::from(array),
            timestamp,
            ethernet: ptr::null(),
        };

        packet.decode();

        return Rc::new(packet);
    }

    fn decode(&mut self) {
        let offset: usize = 0;
        let offset = self.decode_ethernet(offset);
        debug!("{} -------------------- offset", offset);
    }


    fn decode_ethernet(&mut self, offset: usize) -> usize {
        self.ethernet = self.data.as_ptr() as *const EthernetHdr;
        mem::size_of::<EthernetHdr>()
    }
}
