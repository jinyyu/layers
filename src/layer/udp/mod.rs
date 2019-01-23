pub mod dissector;

pub use self::dissector::*;

#[repr(C, packed)]
struct UDPHeader {
    src_port: u16,
    dst_port: u16,
    len: u16,
    checksum: u16,
}
