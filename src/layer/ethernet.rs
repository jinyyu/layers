const ETHERNET_TYPE_PUP: u16 = 0x0200; /* PUP protocol */
const ETHERNET_TYPE_IP: u16 = 0x0800;
const ETHERNET_TYPE_ARP: u16 = 0x0806;
const ETHERNET_TYPE_BRIDGE: u16 = 0x6558; /* transparant ethernet bridge (GRE) */
const ETHERNET_TYPE_REVARP: u16 = 0x8035;
const ETHERNET_TYPE_EAPOL: u16 = 0x888e;
const ETHERNET_TYPE_IPV6: u16 = 0x86dd;
const ETHERNET_TYPE_IPX: u16 = 0x8137;
const ETHERNET_TYPE_PPPOE_DISC: u16 = 0x8863; /* discovery stage */
const ETHERNET_TYPE_PPPOE_SESS: u16 = 0x8864;/* session stage */
const ETHERNET_TYPE_8021AD: u16 = 0x88a8;
const ETHERNET_TYPE_8021AH: u16 = 0x88e7;
const ETHERNET_TYPE_8021Q: u16 = 0x8100;
const ETHERNET_TYPE_LOOP: u16 = 0x9000;
const ETHERNET_TYPE_8021QINQ: u16 = 0x9100;
const ETHERNET_TYPE_ERSPAN: u16 = 0x88BE;
const ETHERNET_TYPE_DCE: u16 = 0x8903;/* Data center ethernet*/


pub fn ethernet_type_string(value: u16) -> &'static str {
    match value {
        ETHERNET_TYPE_PUP => "PUP",
        ETHERNET_TYPE_IP => "IP",
        ETHERNET_TYPE_ARP => "ARP",
        ETHERNET_TYPE_BRIDGE => "BRIDGE",
        ETHERNET_TYPE_REVARP => "REVARP",
        ETHERNET_TYPE_EAPOL => "EAPOL",
        ETHERNET_TYPE_IPV6 => "ETHERNET_TYPE_IPV6",
        ETHERNET_TYPE_IPX => "IPX",
        ETHERNET_TYPE_PPPOE_DISC => "DISC",
        ETHERNET_TYPE_PPPOE_SESS => "SESS",
        ETHERNET_TYPE_8021AD => "8021AD",
        ETHERNET_TYPE_8021AH => "8021AH",
        ETHERNET_TYPE_8021Q => "8021Q",
        ETHERNET_TYPE_LOOP => "LOOP",
        ETHERNET_TYPE_8021QINQ => "8021QINQ",
        ETHERNET_TYPE_ERSPAN => "ERSPAN",
        ETHERNET_TYPE_DCE => "DCE",
        _ => "Unknown"
    }
}


#[repr(C)]
pub struct EthernetHdr {
    pub eth_dst: [u8; 6],
    pub eth_src: [u8; 6],
    pub eth_type: u16,
}