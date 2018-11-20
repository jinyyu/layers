use libc::{c_int, c_uint, c_char};
use std::ffi::CString;
use std::ffi::CStr;
use std::borrow::Cow;
const AF_INET: u32 = 2; //* IP protocol family

extern "C" {
    pub fn htonl(hostlong: u32) -> u32;
    pub fn htons(hostshort: u16) -> u16;
    pub fn ntohl(netlong: u32) -> u32;
    pub fn ntohs(netshort: u16) -> u16;
    fn inet_ntop(af: c_int, src: *const c_char, dst: *mut c_char, size: u32) -> *const c_char;
}

pub fn ip_to_string(ip: u32) -> String {
    let mut array: [u8; 16] = [0; 16];
    let raw: *const u32 = &ip as *const u32;
    unsafe {
        inet_ntop(AF_INET as c_int, raw as *const c_char, array.as_mut_ptr() as *mut c_char, 16);
        let c_str = CStr::from_bytes_with_nul_unchecked(&array);
        return c_str.to_string_lossy().into_owned();
    }
}


