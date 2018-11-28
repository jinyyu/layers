use libc::c_char;
use std::ptr;

#[link(name = "ndpi")]
extern "C" {
    fn ndpi_init_detection_module() -> *const c_char;
    fn ndpi_exit_detection_module(handle: *const c_char);
}


pub struct Detector {
    handle: *const c_char,
}

impl Detector {
    pub fn new() -> Detector {
        unsafe {
            Detector {
                handle: ndpi_init_detection_module(),
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