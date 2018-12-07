use crate::inet;
use std::cmp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IPProto(pub u8);

impl IPProto {
    pub const IP: IPProto = IPProto(0); /* Dummy protocol for TCP.  */
    pub const ICMP: IPProto = IPProto(1); /* Internet Control Message Protocol.  */
    pub const IGMP: IPProto = IPProto(2); /* Internet Group Management Protocol. */
    pub const IPIP: IPProto = IPProto(4); /* IPIP tunnels (older KA9Q tunnels use 94).  */
    pub const TCP: IPProto = IPProto(6); /* Transmission Control Protocol.  */
    pub const EGP: IPProto = IPProto(8); /* Exterior Gateway Protocol.  */
    pub const PUP: IPProto = IPProto(12); /* PUP protocol.  */
    pub const UDP: IPProto = IPProto(17); /* User Datagram Protocol.  */
    pub const IDP: IPProto = IPProto(22); /* XNS IDP protocol.  */
    pub const TP: IPProto = IPProto(29); /* SO Transport Protocol Class 4.  */
    pub const DCCP: IPProto = IPProto(33); /* Datagram Congestion Control Protocol.  */
    pub const IPV6: IPProto = IPProto(41); /* IPv6 header.  */
    pub const RSVP: IPProto = IPProto(46); /* Reservation Protocol.  */
    pub const GRE: IPProto = IPProto(47); /* General Routing Encapsulation.  */
    pub const ESP: IPProto = IPProto(50); /* encapsulating security payload.  */
    pub const AH: IPProto = IPProto(51); /* authentication header.  */
    pub const MTP: IPProto = IPProto(92); /* Multicast Transport Protocol.  */
    pub const BEETPH: IPProto = IPProto(94); /* IP option pseudo header for BEET.  */
    pub const ENCAP: IPProto = IPProto(98); /* Encapsulation Header.  */
    pub const PIM: IPProto = IPProto(103); /* Protocol Independent Multicast.  */
    pub const COMP: IPProto = IPProto(108); /* Compression Header Protocol.  */
    pub const SCTP: IPProto = IPProto(132); /* Stream Control Transmission Protocol.  */
    pub const UDPLITE: IPProto = IPProto(136); /* UDP-Lite protocol.  */
    pub const RAW: IPProto = IPProto(255); /* Raw IP packets.  */

    pub fn to_string(self) -> &'static str {
        match self {
            IPProto::IP => "IP",
            IPProto::ICMP => "ICMP",
            IPProto::IGMP => "IGMP",
            IPProto::IPIP => "IPIP",
            IPProto::TCP => "TCP",
            IPProto::EGP => "EGP",
            IPProto::PUP => "PUP",
            IPProto::UDP => "UDP",
            IPProto::IDP => "IDP",
            IPProto::TP => "TP",
            IPProto::DCCP => "DCCP",
            IPProto::IPV6 => "IPV6",
            IPProto::RSVP => "RSVP",
            IPProto::GRE => "GRE",
            IPProto::ESP => "ESP",
            IPProto::AH => "AH",
            IPProto::MTP => "MTP",
            IPProto::BEETPH => "BEETPH",
            IPProto::ENCAP => "ENCAP",
            IPProto::PIM => "PIM ",
            IPProto::COMP => "COMP",
            IPProto::SCTP => "SCTP",
            IPProto::UDPLITE => "UDPLITE",
            IPProto::RAW => "RAW",
            _ => "Unknown",
        }
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

    #[inline]
    pub fn total_length(&self) -> u16 {
        unsafe { inet::ntohs(self.len) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StreamID {
    min_ip: u32,
    max_ip: u32,
    min_port: u16,
    max_port: u16,
}

impl StreamID {
    pub fn new(client_ip: u32, server_ip: u32, client_port: u16, server_port: u16) -> StreamID {
        StreamID {
            min_ip: cmp::min(client_ip, server_ip),
            max_ip: cmp::max(client_ip, server_ip),
            min_port: cmp::min(client_port, server_port),
            max_port: cmp::max(client_port, server_port),
        }
    }
}
