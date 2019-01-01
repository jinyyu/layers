use gmime_sys;
use gobject_2_0_sys;
use libc::{c_char, c_void};
use std::ffi::{CStr, CString};
use std::ptr;
use std::slice;

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
            _content: *mut gmime_sys::GMimeContentType,
            _user: *mut c_char,
        ),
        _user: *mut c_char,
    );
}

pub struct MimeParser {
    stream: *mut gmime_sys::GMimeStream,
    root_msg: *mut c_char,
    file_data_callback: Option<Box<FileDataCallback>>,
}

type FileDataCallback = Fn(&[u8], bool, String, String);

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
            file_data_callback: None,
        }
    }

    pub fn parse(&mut self, callback: Box<FileDataCallback>) -> Result<(), ()> {
        self.file_data_callback = Some(callback);
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

    fn on_file_data_callback(
        &self,
        data: &[u8],
        is_test: bool,
        filename: String,
        mime_type: String,
    ) {
        match self.file_data_callback.as_ref() {
            Some(cb) => {
                cb(data, is_test, filename, mime_type);
            }
            None => {}
        };
    }

    extern "C" fn on_file_data(
        data: *const c_char,
        len: u32,
        is_text: bool,
        filename: *const c_char,
        content: *mut gmime_sys::GMimeContentType,
        user: *mut c_char,
    ) {
        unsafe {
            let name;
            if filename != ptr::null_mut() {
                let filename = CStr::from_ptr(filename);
                name = filename.to_string_lossy().into_owned();
            } else {
                name = String::new();
            }

            let mime_type;
            if content != ptr::null_mut() {
                let raw = gmime_sys::g_mime_content_type_get_mime_type(content);
                mime_type = CString::from_raw(raw as *mut c_char)
                    .to_string_lossy()
                    .into_owned();
            } else {
                mime_type = String::new();
            }

            let this = &*(user as *mut MimeParser);
            let date = slice::from_raw_parts(data as *const u8, len as usize);
            this.on_file_data_callback(date, is_text, name, mime_type);
        }
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
