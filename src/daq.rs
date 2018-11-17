use std::ptr;
use std::os::raw::c_char;
use std::ffi::CString;
use libc::{c_int, timeval, c_uint};

use config;

#[repr(C)]
pub struct pcap_t {}

#[repr(C)]
pub struct DAQ {
    handle: *mut pcap_t,
}

extern "C" fn loop_callback(this: *const DAQ, packet: *const pcap_pkthdr, bytes: *const c_char) {
    unsafe {
        println!("---------------------{} libpcap", (*packet).ts)
    }
}

#[link(name = "pcap")]
extern "C" {
    fn pcap_create(device: *const c_char, error: *mut c_char) -> *mut pcap_t;
    fn pcap_set_snaplen(handle: *mut pcap_t, snaplen: c_int) -> c_int;
    fn pcap_set_buffer_size(handle: *mut pcap_t, buffer_size: c_int) -> c_int;
    fn pcap_set_promisc(handle: *mut pcap_t, promisc: c_int) -> c_int;
    fn pcap_activate(handle: *mut pcap_t) -> c_int;
    fn pcap_close(handle: *mut pcap_t);
    fn pcap_loop(handle: *mut pcap_t, count: c_int, cb: extern fn(*const DAQ, *const pcap_pkthdr, *const c_char)) -> c_int;
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct pcap_pkthdr {
    pub ts: timeval,
    pub caplen: c_uint,
    pub len: c_uint,
}


impl DAQ {
    pub fn run(&self) {
        unsafe {
            let ret = pcap_loop(&mut *self.handle, -1, loop_callback);
            info!("pcap_loop exit {}", ret);
        }
    }
}


pub fn init(conf: &config::Configure) -> Option<Box<DAQ>> {
    let handle = open_device(&conf.interface);
    match handle {
        Some(h) => {
            let daq = DAQ { handle: h };
            return Some(Box::new(daq));
        }
        None => None,
    }
}

fn open_device(device: &str) -> Option<*mut pcap_t> {
    let device = CString::new(device.as_bytes()).unwrap();
    let mut buff: Vec<c_char> = Vec::with_capacity(256);
    let errbuf = buff.as_mut_ptr();
    unsafe {
        let handle = pcap_create(device.as_ptr(), errbuf);
        if handle.is_null() {
            error!("pcap_create error {}", CString::from_raw(errbuf).to_str().unwrap());
            return None;
        }

        //64k
        let ret = pcap_set_snaplen(handle, 1024 * 64);
        if ret != 0 {
            error!("pcap_set_snaplen error");
            pcap_close(handle);
            return None;
        }

        //500M
        let ret = pcap_set_buffer_size(handle, 500 * 1024 * 1024);
        if ret != 0 {
            error!("pcap_set_buffer_size error");
            pcap_close(handle);
            return None;
        }

        //500M
        let ret = pcap_set_promisc(handle, 1);
        if ret != 0 {
            error!("pcap_set_promisc error");
            pcap_close(handle);
            return None;
        }

        let ret = pcap_activate(handle);

        if ret != 0 {
            error!("pcap_activate error");
            pcap_close(handle);
            return None;
        }
        return Some(handle);
    }
}

