#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EthernetType(pub u16);

impl EthernetType {
    pub const PUP: EthernetType = EthernetType(0x0200); /* PUP protocol */
    pub const IP: EthernetType = EthernetType(0x0800);
    pub const ARP: EthernetType = EthernetType(0x0806);
    pub const BRIDGE: EthernetType = EthernetType(0x6558); /* transparant ethernet bridge (GRE) */
    pub const REVARP: EthernetType = EthernetType(0x8035);
    pub const EAPOL: EthernetType = EthernetType(0x888e);
    pub const IPV6: EthernetType = EthernetType(0x86dd);
    pub const IPX: EthernetType = EthernetType(0x8137);
    pub const PPPOE_DISC: EthernetType = EthernetType(0x8863); /* discovery stage */
    pub const PPPOE_SESS: EthernetType = EthernetType(0x8864); /* session stage */
    pub const T8021AD: EthernetType = EthernetType(0x88a8);
    pub const T8021AH: EthernetType = EthernetType(0x88e7);
    pub const VLAN: EthernetType = EthernetType(0x8100);
    pub const LOOP: EthernetType = EthernetType(0x9000);
    pub const T8021QINQ: EthernetType = EthernetType(0x9100);
    pub const ERSPAN: EthernetType = EthernetType(0x88BE);
    pub const DCE: EthernetType = EthernetType(0x8903); /* Data center ethernet*/

    pub fn ethernet_type_string(value: EthernetType) -> &'static str {
        match value {
            EthernetType::PUP => "PUP",
            EthernetType::IP => "IP",
            EthernetType::ARP => "ARP",
            EthernetType::BRIDGE => "BRIDGE",
            EthernetType::REVARP => "REVARP",
            EthernetType::EAPOL => "EAPOL",
            EthernetType::IPV6 => "IPV6",
            EthernetType::IPX => "IPX",
            EthernetType::PPPOE_DISC => "DISC",
            EthernetType::PPPOE_SESS => "SESS",
            EthernetType::T8021AD => "8021AD",
            EthernetType::T8021AH => "8021AH",
            EthernetType::VLAN => "VLAN",
            EthernetType::LOOP => "LOOP",
            EthernetType::T8021QINQ => "8021QINQ",
            EthernetType::ERSPAN => "ERSPAN",
            EthernetType::DCE => "DCE",
            _ => "Unknown",
        }
    }
}

#[repr(C, packed)]
pub struct EthernetHeader {
    pub eth_dst: [u8; 6],
    pub eth_src: [u8; 6],
    pub eth_type: u16,
}

impl EthernetHeader {
    pub fn src_mac(&self) -> String {
        return format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            (self.eth_src)[0],
            (self.eth_src)[1],
            (self.eth_src)[2],
            (self.eth_src)[3],
            (self.eth_src)[4],
            (self.eth_src)[5]
        );
    }

    pub fn dst_mac(&self) -> String {
        return format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            (self.eth_dst)[0],
            (self.eth_dst)[1],
            (self.eth_dst)[2],
            (self.eth_dst)[3],
            (self.eth_dst)[4],
            (self.eth_dst)[5]
        );
    }
}
