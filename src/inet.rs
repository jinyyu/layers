

extern "C" {
    pub fn htonl(hostlong:u32) -> u32;
    pub fn htons(hostshort:u16)->u16;
    pub fn ntohl(netlong:u32) ->u32;
    pub fn ntohs(netshort:u16)->u16;
}
