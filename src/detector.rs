use libc::c_char;
use libc;
use std::ptr;
use layer::TCPHeader;
use std::ffi::CStr;
use std::mem;
use std::sync::Arc;
use config::Configure;
use layer::tcp::dissector::{TCPDissectorAllocator, TCPDissector};
use std::rc::Rc;
use std::cell::RefCell;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Proto {
    pub master_id: u16,
    pub app_id: u16,
    pub category: u32,
}

impl Proto {
    pub const UNKNOWN: u16 = 0;

    pub fn new() -> Proto {
        Proto {
            master_id: 0,
            app_id: 0,
            category: 0,
        }
    }

    #[inline]
    pub fn success(&self) -> bool {
        self.app_id != Proto::UNKNOWN || self.master_id != Proto::UNKNOWN
    }
}


#[link(name = "layerscpp")]
#[link(name = "ndpi")]
extern "C" {
    pub fn ndpi_detection_process_packet(ctx: *const c_char,
                                         flow: *const c_char,
                                         packet: *const c_char,
                                         packet_len: u16,
                                         tm: u64,
                                         src_id: *const c_char,
                                         dst_id: *const c_char) -> Proto;

    pub fn ndpi_detection_giveup(ctx: *const c_char, flow: *const c_char) -> Proto;

    pub fn ndpi_guess_undetected_protocol(ctx: *const c_char,
                                          proto: u8,
                                          src_ip: u32,
                                          src_port: u16,
                                          dst_ip: u16,
                                          dst_port: u16) -> Proto;



    fn ndpi_protocol2name(ctx: *const c_char,
                          proto: Proto,
                          buf: *mut c_char,
                          len: u32) -> *const c_char;


    pub fn init_ndpi_ctx() -> *const c_char;
    pub fn free_ndpi_ctx(ctx: *const c_char);

    pub fn new_ndpi_flow() -> *const c_char;
    pub fn free_ndpi_flow(ctx: *const c_char);


    pub fn new_ndpi_flow_id() -> *const c_char;
    pub fn free_ndpi_flow_id(ctx: *const c_char);
}


pub struct Detector {
    ctx: *const c_char,
    tcp_dissector_allocator: TCPDissectorAllocator,
}

impl Detector {
    pub fn new(conf: Arc<Configure>) -> Detector {
        unsafe {
            Detector {
                ctx: init_ndpi_ctx(),
                tcp_dissector_allocator: TCPDissectorAllocator::new(conf.clone()),
            }
        }
    }


    #[inline]
    pub fn detect(&self,
                  flow: *const c_char,
                  ip_layer: *const c_char,
                  ip_layer_len: u16,
                  tm: u64,
                  src_id: *const c_char,
                  dst_id: *const c_char) -> Proto {
        unsafe {
            ndpi_detection_process_packet(self.ctx, flow, ip_layer as *const c_char, ip_layer_len, tm, src_id, dst_id)
        }
    }

    pub fn protocol_name(&self, proto: &Proto) -> String {
        let mut array: [u8; 16] = [0; 16];
        let c_str;
        unsafe {
            ndpi_protocol2name(self.ctx, *proto, array.as_mut_ptr() as *mut i8, 16);
            c_str = CStr::from_bytes_with_nul_unchecked(&array);
        }
        return c_str.to_string_lossy().into_owned();
    }

    pub fn alloc_tcp_dissector(&self, proto: &Proto) -> Rc<RefCell<TCPDissector>> {
        self.tcp_dissector_allocator.alloc_dissector(proto)
    }
}


impl Drop for Detector {
    fn drop(&mut self) {
        debug!("detector cleanup");
        unsafe {
            free_ndpi_ctx(self.ctx);
        }
    }
}