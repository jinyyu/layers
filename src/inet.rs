use libc::{c_int, c_uint, c_char};

extern "C" {
    pub fn htonl(hostlong: u32) -> u32;
    pub fn htons(hostshort: u16) -> u16;
    pub fn ntohl(netlong: u32) -> u32;
    pub fn ntohs(netshort: u16) -> u16;
    pub fn inet_ntop(af: c_int, src: *const c_char, dst: * mut c_char, size: u32) -> *const c_char;
}
