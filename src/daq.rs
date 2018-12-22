use crate::config;
use crate::packet::Packet;
use libc::{c_char, c_int, c_uint};
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::sync::{Arc, Mutex};

#[repr(C)]
pub struct DAQ {
    handle: *const c_char,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Timeval {
    sec: u64,
    usec: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct PacketHeader {
    pub ts: Timeval,
    pub caplen: c_uint,
    pub len: c_uint,
}

extern "C" fn loop_callback(ctx: *mut c_char, packet: *const PacketHeader, bytes: *const c_char) {
    unsafe {
        let tm = (*packet).ts.sec * 1000 * 1000 + (*packet).ts.usec;
        let p = Packet::new(tm, bytes as *const u8, (*packet).len as usize);
        if !p.valid() {
            debug!("invalid packet 0b{:b}", p.flag);
        } else {
            let callback = ctx as *const &Fn(Arc<Packet>);
            (*callback)(p);
        }
    };
}

//pfring
#[link(name = "pcap")]
//#[link(name = "pfring")]
extern "C" {
    fn pcap_create(_device: *const c_char, _error: *mut c_char) -> *const c_char;
    fn pcap_set_snaplen(_handle: *const c_char, _snaplen: c_int) -> c_int;
    fn pcap_set_buffer_size(_handle: *const c_char, _buffer_size: c_int) -> c_int;
    fn pcap_set_promisc(_handle: *const c_char, _promisc: c_int) -> c_int;
    fn pcap_activate(_handle: *const c_char) -> c_int;
    fn pcap_close(_handle: *const c_char);
    fn pcap_loop(
        _handle: *const c_char,
        _count: c_int,
        _cb: extern "C" fn(ctx: *mut c_char, *const PacketHeader, *const c_char),
        _ctx: *const c_char,
    ) -> c_int;

    fn pcap_breakloop(_handle: *const c_char);
}

static mut PCAP_HANDLE: u64 = 0;

impl DAQ {
    pub fn shutdown() {
        unsafe {
            if PCAP_HANDLE > 0 {
                let handle = mem::transmute::<u64, *const c_char>(PCAP_HANDLE);
                pcap_breakloop(handle);
            }
        }
    }

    pub fn run(&self, cb: &Fn(Arc<Packet>)) {
        info!("pcap_loop");
        unsafe {
            pcap_loop(
                self.handle,
                -1,
                loop_callback,
                &cb as *const &Fn(Arc<Packet>) as *const c_char,
            );
        }
        info!("pcap_loop exit ");
    }
}

impl Drop for DAQ {
    fn drop(&mut self) {
        debug!("daq cleanup");
        unsafe {
            pcap_close(self.handle);
        }
    }
}

pub fn init(conf: Arc<config::Configure>) -> Arc<DAQ> {
    let handle = open_device(&conf.interface);
    match handle {
        Some(h) => {
            let daq = DAQ { handle: h };
            return Arc::new(daq);
        }
        None => {
            panic!("init daq error");
        }
    }
}

fn open_device(device: &str) -> Option<*const c_char> {
    let device = CString::new(device.as_bytes()).unwrap();
    let mut buff: Vec<c_char> = Vec::with_capacity(256);
    let buffer = buff.as_mut_ptr();
    unsafe {
        let handle = pcap_create(device.as_ptr() as *const c_char, buffer);

        if handle.is_null() {
            error!(
                "pcap_create error {}",
                CString::from_raw(buffer as *mut c_char).to_str().unwrap()
            );
            return None;
        }

        let pcap_handle = mem::transmute::<*const c_char, u64>(handle);
        PCAP_HANDLE = pcap_handle;

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
