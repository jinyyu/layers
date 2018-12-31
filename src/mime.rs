use gmime_sys;
use gobject_2_0_sys;
use libc::{c_char, c_void};
use std::ptr;

extern "C" {
    fn new_mime_message(_msg: *mut gmime_sys::GMimeObject) -> *mut c_char;
    fn delete_mime_message(_msg: *mut c_char);
    fn mime_message_walk(
        _msg: *mut c_char,
        _cb: extern "C" fn(
            _data: *const c_char,
            _len: u32,
            _is_text: bool,
            _filename: *const c_char,
            _content: *const gmime_sys::GMimeContentType,
            _user: *mut c_char,
        ),
        _user: *mut c_char,
    );
}

pub struct MimeParser {
    stream: *mut gmime_sys::GMimeStream,
    root_msg: *mut c_char,
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
        MimeParser {
            stream,
            root_msg: ptr::null_mut(),
        }
    }

    pub fn parse(&mut self) -> Result<(), ()> {
        unsafe {
            let parser = gmime_sys::g_mime_parser_new_with_stream(self.stream);
            gmime_sys::g_mime_parser_set_format(parser, gmime_sys::GMIME_FORMAT_MESSAGE);

            let msg = gmime_sys::g_mime_parser_construct_message(
                parser,
                ptr::null_mut() as *mut gmime_sys::GMimeParserOptions,
            );
            if msg != ptr::null_mut() && (*msg).mime_part != ptr::null_mut() {
                self.root_msg = new_mime_message((*msg).mime_part);

                let this = self as *mut MimeParser as *mut c_char;

                mime_message_walk(self.root_msg, MimeParser::on_file_data, this);
            }

            if msg != ptr::null_mut() {
                gobject_2_0_sys::g_object_unref(msg as *mut c_void);
            }

            gobject_2_0_sys::g_object_unref(parser as *mut c_void);
        }

        Ok(())
    }

    extern "C" fn on_file_data(
        data: *const c_char,
        len: u32,
        is_text: bool,
        filename: *const c_char,
        content: *const gmime_sys::GMimeContentType,
        user: *mut c_char,
    ) {
        let this = unsafe { &*(user as *mut MimeParser) };
    }
}

impl Drop for MimeParser {
    fn drop(&mut self) {
        unsafe {
            if self.root_msg != ptr::null_mut() {
                delete_mime_message(self.root_msg);
            }
            gobject_2_0_sys::g_object_unref(self.stream as *mut c_void);
        }
    }
}
