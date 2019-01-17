
#[repr(C,packed)]
pub struct VlanHeader {
    pub vlan_id:u16,
    pub eth_type: u16,
}