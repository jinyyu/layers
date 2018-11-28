use libc::c_char;
use libc;
use std::ptr;

#[link(name = "ndpi")]
extern "C" {
    fn ndpi_exit_detection_module(handle: *const c_char);
    fn ndpi_flow_malloc(size: isize) -> *const c_char;
    pub fn ndpi_free_flow(flow: *const c_char);
}


#[link(name = "layerscpp")]
extern "C" {
    fn alloc_ndpi() -> *const c_char;
    fn ndpi_flow_struct_size() -> u32;
}


pub struct Detector {
    handle: *const c_char,
    ndpi_flow_struct_size: usize,
}

impl Detector {
    pub fn new() -> Detector {
        unsafe {
            let handle = alloc_ndpi();

            Detector {
                handle,
                ndpi_flow_struct_size: ndpi_flow_struct_size() as usize,
            }
        }
    }

    pub fn drop(&mut self) {
        debug!("detector cleanup");
        unsafe {
            ndpi_exit_detection_module(self.handle);
        }
    }

    #[inline]
    pub fn new_ndpi_flow(&self) -> *const c_char {
        unsafe {
            ndpi_flow_malloc(self.ndpi_flow_struct_size as isize)
        }
    }
}