pub struct IPV4Header {
    //version & header length
    pub version: u8,

    //type of service
    pub tos: u8,

    // including header and data
    pub len: u16,

    //id
    pub id: u16,

    //frag offset
    pub off: u16,

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