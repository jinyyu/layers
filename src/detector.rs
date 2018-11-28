use libc::c_char;
use std::ptr;

#[link(name = "ndpi")]
extern "C" {
    fn ndpi_exit_detection_module(handle: *const c_char);
}


#[link(name = "layerscpp")]
extern "C" {
    fn alloc_ndpi() -> *const c_char;
}





pub struct Detector {
    handle: *const c_char,
}

impl Detector {
    pub fn new() -> Detector {
        unsafe {
            let handle = alloc_ndpi();

            Detector {
                handle,
            }
        }
    }

    pub fn drop(&mut self) {
        debug!("detector cleanup");
        unsafe {
            ndpi_exit_detection_module(self.handle);
        }
    }
}