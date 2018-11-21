use libc::{c_int, c_char};
use std::ffi::CStr;
use std::mem;

const AF_INET: u32 = 2; //* IP protocol family

#[link(name = "layerscpp", kind = "static")]
extern "C" {
    pub fn htonl(hostlong: u32) -> u32;
    pub fn htons(hostshort: u16) -> u16;
    pub fn ntohl(netlong: u32) -> u32;
    pub fn ntohs(netshort: u16) -> u16;
    fn inet_ntop(af: c_int, src: *const c_char, dst: *mut c_char, size: u32) -> *const c_char;
    fn layers_checksum(buf: *const c_char, size: usize) -> u16;
}

pub fn ip_to_string(ip: u32) -> String {
    let mut array: [u8; 16] = [0; 16];
    let c_str;
    unsafe {
        let raw = mem::transmute::<*const u32, *mut c_char>(&ip);
        inet_ntop(AF_INET as c_int, raw, array.as_mut_ptr() as *mut c_char, 16);
        c_str = CStr::from_bytes_with_nul_unchecked(&array);
    }
    return c_str.to_string_lossy().into_owned();
}


pub fn checksum(buf: *const c_char, size: usize) -> u16 {
    unsafe {
        return layers_checksum(buf, size);
    }
}

