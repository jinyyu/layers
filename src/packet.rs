use std::slice;
use std::rc::Rc;

pub struct Packet {
    pub timestamp: u64,
    pub data: Vec<u8>,
}


impl Packet {
    pub fn new(timestamp: u64, data: *const u8, size: usize) -> Rc<Packet> {
        debug!("data len = {}", size);
        let array = unsafe { slice::from_raw_parts(data, size) };

        let mut packet = Packet {
            data: Vec::from(array),
            timestamp,
        };

        packet.decode();

        return Rc::new(packet);
    }

    fn decode(&mut self) {

    }
}
