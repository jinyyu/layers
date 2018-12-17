use gmime_sys;

pub struct MimeParser {}

impl MimeParser {
    pub fn init() {
        unsafe { gmime_sys::g_mime_init(); }
    }
    pub fn new(data: &[u8]) -> MimeParser {
        MimeParser {}
    }
}