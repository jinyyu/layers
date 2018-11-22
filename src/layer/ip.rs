
pub const IPPROTO_IP:u8 = 0;	   /* Dummy protocol for TCP.  */
pub const IPPROTO_ICMP:u8 = 1;	   /* Internet Control Message Protocol.  */
pub const IPPROTO_IGMP:u8 = 2;	   /* Internet Group Management Protocol. */
pub const IPPROTO_IPIP:u8 = 4;	   /* IPIP tunnels (older KA9Q tunnels use 94).  */
pub const IPPROTO_TCP:u8 =  6;	   /* Transmission Control Protocol.  */
pub const IPPROTO_EGP:u8 =  8;	   /* Exterior Gateway Protocol.  */
pub const IPPROTO_PUP:u8 = 12;	   /* PUP protocol.  */
pub const IPPROTO_UDP:u8 = 17;	   /* User Datagram Protocol.  */
pub const IPPROTO_IDP:u8 = 22;	   /* XNS IDP protocol.  */
pub const IPPROTO_TP:u8 = 29;	   /* SO Transport Protocol Class 4.  */
pub const IPPROTO_DCCP:u8 = 33;	   /* Datagram Congestion Control Protocol.  */
pub const IPPROTO_IPV6:u8 = 41;     /* IPv6 header.  */
pub const IPPROTO_RSVP:u8 = 46;	   /* Reservation Protocol.  */
pub const IPPROTO_GRE :u8= 47;	   /* General Routing Encapsulation.  */
pub const IPPROTO_ESP:u8 = 50;      /* encapsulating security payload.  */
pub const IPPROTO_AH:u8 = 51;      /* authentication header.  */
pub const IPPROTO_MTP:u8 = 92;	   /* Multicast Transport Protocol.  */
pub const IPPROTO_BEETPH:u8 = 94;   /* IP option pseudo header for BEET.  */
pub const IPPROTO_ENCAP:u8 = 98;	   /* Encapsulation Header.  */
pub const IPPROTO_PIM :u8= 103;   /* Protocol Independent Multicast.  */
pub const IPPROTO_COMP:u8 = 108;	   /* Compression Header Protocol.  */
pub const IPPROTO_SCTP:u8 = 132;	   /* Stream Control Transmission Protocol.  */
pub const IPPROTO_UDPLITE:u8 = 136; /* UDP-Lite protocol.  */
pub const IPPROTO_RAW:u8 = 255;	   /* Raw IP packets.  */


pub fn ip_type_string(value: u8) -> &'static str {
    match value {
        IPPROTO_IP => "IP",
        IPPROTO_ICMP => "ICMP",
        IPPROTO_IGMP => "IGMP",
        IPPROTO_IPIP => "IPIP",
        IPPROTO_TCP => "TCP",
        IPPROTO_EGP => "EGP",
        IPPROTO_PUP => "PUP",
        IPPROTO_UDP => "UDP",
        IPPROTO_IDP => "IDP",
        IPPROTO_TP => "TP",
        IPPROTO_DCCP => "DCCP",
        IPPROTO_IPV6 => "IPV6",
        IPPROTO_RSVP => "RSVP",
        IPPROTO_GRE => "GRE",
        IPPROTO_ESP => "ESP",
        IPPROTO_AH => "AH",
        IPPROTO_MTP => "MTP",
        IPPROTO_BEETPH => "BEETPH",
        IPPROTO_ENCAP => "ENCAP",
        IPPROTO_PIM => "PIM ",
        IPPROTO_COMP => "COMP",
        IPPROTO_SCTP => "SCTP",
        IPPROTO_UDPLITE => "UDPLITE",
        IPPROTO_RAW => "RAW",
        _ => "Unknown"
    }
}


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
