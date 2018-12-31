use gmime_sys;
use gobject_2_0_sys;
use libc::c_void;

pub struct MimeParser {
    stream: *mut gmime_sys::GMimeStream,
}

impl MimeParser {
    pub fn init() {
        unsafe { gmime_sys::g_mime_init() }
    }

    pub fn shutdown() {
        unsafe { gmime_sys::g_mime_shutdown() }
    }

    pub fn new(stream: *mut gmime_sys::GMimeStream) -> MimeParser {
        unsafe {
            gobject_2_0_sys::g_object_ref(stream as *mut c_void);
        }
        MimeParser { stream }
    }

    pub fn parse(&mut self) -> Result<(), ()> {
        Ok(())
    }
}

impl Drop for MimeParser {
    fn drop(&mut self) {
        unsafe {
            gobject_2_0_sys::g_object_ref(self.stream as *mut c_void);
        }
    }
}
