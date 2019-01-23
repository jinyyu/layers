pub mod dissector;

pub use self::dissector::*;

#[repr(C, packed)]
pub struct UDPHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub len: u16,
    pub checksum: u16,
}
