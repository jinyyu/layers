#[repr(C)]
pub struct IPV4Header {
    //version & header length
    pub version_length: u8,

    //type of service
    pub tos: u8,

    // including header and data
    pub len: u16,

    //id
    pub id: u16,

    //Flags & frag offset
    pub flag_offset: u16,

    //time to live
    pub ttl: u8,

    //protocol (tcp, udp, etc)
    pub proto: u8,

    //checksum
    pub checksum: u16,

    //source address
    pub src: u32,

    //destination address
    pub dst: u32,
}

impl IPV4Header {
    #[inline]
    pub fn version(&self) -> u8 {
        return (self.version_length & 0xF0) >> 4;
    }

    #[inline]
    pub fn header_len(&self) -> u8 {
        return (self.version_length & 0x0F) << 2;
    }
}
