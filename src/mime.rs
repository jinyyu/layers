pub struct MimeParser {}

impl MimeParser {
    pub fn init() {
        unsafe { gmime_sys::g_mime_init() }
    }

    pub fn shutdown() {
        unsafe { gmime_sys::g_mime_shutdown() }
    }
}
