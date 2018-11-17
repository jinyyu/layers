

#[repr(C)]
pub struct EthernetHdr {
    pub eth_dst: [u8; 6],
    pub eth_src: [u8; 6],
    pub eth_type: u16,
}