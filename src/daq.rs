use std::ptr;
use std::os::raw::c_char;
use std::ffi::CString;
use libc::c_int;

use config;

pub struct pcap_t {}

#[link(name = "pcap", kind = "static")]
#[link(name = "pfring", kind = "static")]
extern "C" {
    fn pcap_create(device: *const c_char, error: *mut c_char) -> * mut pcap_t;
    fn pcap_set_snaplen(handle: * mut pcap_t, snaplen: c_int) -> c_int;
    fn pcap_set_buffer_size(handle: * mut pcap_t, buffer_size: c_int) -> c_int;
    fn pcap_set_promisc(handle: * mut pcap_t, promisc: c_int) -> c_int;
    fn pcap_activate(handle: * mut pcap_t) -> c_int;
}

pub struct DAQ {}

pub fn init(conf: &config::Configure) -> Option<DAQ> {
    let mut daq: Option<DAQ> = Option::None;
    let device = CString::new(conf.interface.as_bytes()).unwrap();
    let mut buff: Vec<c_char> = Vec::with_capacity(256);
    let mut errbuf = buff.as_mut_ptr();
    unsafe {
        let handle = pcap_create(device.as_ptr(), errbuf);
        if handle.is_null() {
            panic!("pcap_create error {}", CString::from_raw(errbuf).to_str().unwrap());
        }

        //64k
        let ret = pcap_set_snaplen(handle, 1024 * 64);
        if ret != 0 {
            panic!("pcap_set_snaplen error");
        }

        //500M
        let ret = pcap_set_buffer_size(handle, 500 * 1024 * 1024);
        if ret != 0 {
            panic!("pcap_set_buffer_size error");
        }

        //500M
        let ret = pcap_set_promisc(handle, 1);
        if ret != 0 {
            panic!("pcap_set_promisc error");
        }

        let ret = pcap_activate(handle);

        if ret != 0 {
            panic!("pcap_activate error");
        }
    }
    daq
}

