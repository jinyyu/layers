use std::ptr;
use std::os::raw::c_char;
use std::ffi::CString;
use libc::{c_int, c_uint};
use packet::Packet;
use std::mem;
use layer;

use config;

#[repr(C)]
pub struct PcapHandle {}

#[repr(C)]
pub struct DAQ {
    handle: *mut PcapHandle,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Timeval {
    sec: u64,
    usec: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PacketHeader {
    pub ts: Timeval,
    pub caplen: c_uint,
    pub len: c_uint,
}

static ETHERNET_HEADER_LEN: usize = mem::size_of::<layer::EthernetHdr>();


extern "C" fn loop_callback(this: *const DAQ, packet: *const PacketHeader, bytes: *const c_char) {
    let p;
    unsafe {
        let size = (*packet).caplen as usize;
        if size < ETHERNET_HEADER_LEN {
            debug!("invalid packet {}", size);
            return;
        }
        let tm = (*packet).ts.sec * 1000 * 1000 + (*packet).ts.usec;
        p = Packet::new(tm, bytes as *const u8, size);
    };
    debug!("timestamp = {}", p.timestamp)
}

#[link(name = "pcap")]
extern "C" {
    fn pcap_create(device: *const c_char, error: *mut c_char) -> *mut PcapHandle;
    fn pcap_set_snaplen(handle: *mut PcapHandle, snaplen: c_int) -> c_int;
    fn pcap_set_buffer_size(handle: *mut PcapHandle, buffer_size: c_int) -> c_int;
    fn pcap_set_promisc(handle: *mut PcapHandle, promisc: c_int) -> c_int;
    fn pcap_activate(handle: *mut PcapHandle) -> c_int;
    fn pcap_close(handle: *mut PcapHandle);
    fn pcap_loop(handle: *mut PcapHandle, count: c_int, cb: extern fn(*const DAQ, *const PacketHeader, *const c_char)) -> c_int;
}


impl DAQ {
    pub fn run(&self) {
        info!("pcap start");
        unsafe {
            pcap_loop(&mut *self.handle, -1, loop_callback);
        }
        info!("pcap_loop exit ");
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

fn open_device(device: &str) -> Option<*mut PcapHandle> {
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

