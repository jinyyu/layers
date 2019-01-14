use crate::detector::Detector;
use crate::layer::TCPDissector;
use config::Configure;
use gmime_sys;
use gobject_2_0_sys;
use libc::{c_char, c_void, free, malloc, strlen};
use mime;
use mime::MimeParser;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::slice;

const REQUEST_SETTING: ParserSettings = ParserSettings {
    on_message_begin: on_request_message_begin,
    on_url,
    on_status,
    on_header_field: on_request_header_field,
    on_header_value: on_request_header_value,
    on_headers_complete: on_request_headers_complete,
    on_body: on_request_body,
    on_message_complete: on_request_message_complete,
    on_chunk_header,
    on_chunk_complete,
};

const RESPONSE_SETTING: ParserSettings = ParserSettings {
    on_message_begin: on_response_message_begin,
    on_url,
    on_status,
    on_header_field: on_response_header_field,
    on_header_value: on_response_header_value,
    on_headers_complete: on_response_headers_complete,
    on_body: on_response_body,
    on_message_complete: on_response_message_complete,
    on_chunk_header,
    on_chunk_complete,
};

#[repr(C)]
enum HttpParserType {
    Request,
    Response,
    _Both,
}

#[repr(C)]
struct Parser {
    opaque1: u32,
    nread: u32,
    content_length: u64,

    http_major: u16,
    http_minor: u16,
    status_code: u16,
    opaque2: u16,

    data: *const c_char,
}

type HTTPDataCallback =
    extern "C" fn(_parser: *const Parser, _data: *const c_char, _length: isize) -> i32;

type HTTPCallback = extern "C" fn(_parser: *const Parser) -> i32;

#[repr(C)]
struct ParserSettings {
    on_message_begin: HTTPCallback,
    on_url: HTTPDataCallback,
    on_status: HTTPDataCallback,
    on_header_field: HTTPDataCallback,
    on_header_value: HTTPDataCallback,
    on_headers_complete: HTTPCallback,
    on_body: HTTPDataCallback,
    on_message_complete: HTTPCallback,
    on_chunk_header: HTTPCallback,
    on_chunk_complete: HTTPCallback,
}

extern "C" {
    fn http_parser_init(_parser: *mut Parser, _t: HttpParserType);
    fn http_parser_execute(
        _parser: *const Parser,
        _setting: *const ParserSettings,
        _data: *const c_char,
        _len: isize,
    ) -> isize;

    fn http_errno_description_from_parser(_parser: *const Parser) -> *const c_char;
}

extern "C" fn on_chunk_header(_parser: *const Parser) -> i32 {
    0
}

extern "C" fn on_chunk_complete(_parser: *const Parser) -> i32 {
    0
}

extern "C" fn on_request_message_begin(parser: *const Parser) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };
    this.parse_request = true;
    this.request_headers.clear();
    this.request_content_type.clear();

    0
}

extern "C" fn on_url(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };
    if (this).url.is_empty() {
        let data = unsafe { slice::from_raw_parts(data as *const u8, length as usize) };
        this.url = String::from_utf8_lossy(data).to_lowercase();
    }
    0
}

extern "C" fn on_request_header_field(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };
    let data = unsafe { slice::from_raw_parts(data as *const u8, length as usize) };
    this.request_header = String::from_utf8_lossy(data).to_lowercase();
    0
}

extern "C" fn on_request_header_value(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };
    let data = unsafe { slice::from_raw_parts(data as *const u8, length as usize) };
    let header = this.request_header.clone();
    let value = String::from_utf8_lossy(data).to_lowercase();

    (*this).request_headers.insert(header, value);
    0
}

extern "C" fn on_request_headers_complete(parser: *const Parser) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };

    let result = this.request_headers.get("content-type");
    match result {
        Some(value) => {
            trace!("update request content-type {}", value);
            this.request_content_type = value.clone();
        }
        None => {
            trace!("no Content-Type");
        }
    }

    let c = Configure::singleton();

    this.parse_request = c.is_parse_http_content(&this.request_content_type);

    if !this.parse_request {
        return 0;
    }

    if this.request_stream == ptr::null_mut() {
        this.request_stream = unsafe { gmime_sys::g_mime_stream_mem_new() };
    }
    unsafe {
        gmime_sys::g_mime_stream_reset(this.request_stream);
    }

    let mut string = String::new();

    for (k, v) in this.request_headers.iter() {
        string.push_str(k);
        string.push_str(": ");
        string.push_str(v);
        string.push_str("\r\n");
    }

    string.push_str("\r\n");

    unsafe {
        gmime_sys::g_mime_stream_write(
            this.request_stream,
            string.as_ptr() as *const c_char,
            string.len(),
        );
    }

    0
}

extern "C" fn on_request_body(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };
    if !this.parse_request {
        return 0;
    }
    unsafe {
        gmime_sys::g_mime_stream_write(this.request_stream, data, length as usize);
    }
    0
}

extern "C" fn on_request_message_complete(parser: *const Parser) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };
    if !this.parse_request {
        return 0;
    }

    let stream = this.request_stream;
    this.parse_stream(stream);
    0
}

extern "C" fn on_response_message_begin(parser: *const Parser) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };

    this.parse_response = true;
    this.response_headers.clear();
    this.response_content_type.clear();
    0
}

extern "C" fn on_status(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    unsafe {
        if (*parser).status_code != 200 {
            let s =
                String::from_utf8_lossy(slice::from_raw_parts(data as *const u8, length as usize));
            trace!("http error : {} {}", (*parser).status_code, s);
        } else {
        }
    }
    0
}

extern "C" fn on_response_header_field(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };
    let key = unsafe {
        String::from_utf8_lossy(slice::from_raw_parts(data as *const u8, length as usize))
            .to_lowercase()
    };
    this.response_header = key;
    0
}

extern "C" fn on_response_header_value(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };
    let value = unsafe {
        String::from_utf8_lossy(slice::from_raw_parts(data as *const u8, length as usize))
            .to_lowercase()
    };
    let key = this.response_header.clone();
    this.response_headers.insert(key, value);
    0
}

extern "C" fn on_response_headers_complete(parser: *const Parser) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };

    let result = this.response_headers.get("content-type");
    match result {
        Some(value) => {
            trace!("update response content-type {}", value);
            this.response_content_type = value.clone();
        }
        None => {
            trace!("no content-type");
        }
    }
    let c = Configure::singleton();
    this.parse_response = c.is_parse_http_content(&this.response_content_type);

    if !this.parse_response {
        return 0;
    }

    if this.response_stream == ptr::null_mut() {
        this.response_stream = unsafe { gmime_sys::g_mime_stream_mem_new() };
    }
    unsafe {
        gmime_sys::g_mime_stream_reset(this.response_stream);
    }

    let mut string = String::new();

    for (k, v) in this.response_headers.iter() {
        string.push_str(k);
        string.push_str(": ");
        string.push_str(v);
        string.push_str("\r\n");
    }

    string.push_str("\r\n");

    unsafe {
        gmime_sys::g_mime_stream_write(
            this.response_stream,
            string.as_ptr() as *const c_char,
            string.len(),
        );
    }
    0
}

extern "C" fn on_response_body(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };
    if !this.parse_response {
        return 0;
    }
    unsafe {
        gmime_sys::g_mime_stream_write(this.response_stream, data, length as usize);
    }
    0
}

extern "C" fn on_response_message_complete(parser: *const Parser) -> i32 {
    let this = unsafe { &mut *((*parser).data as *mut HTTPDissector) };
    if !this.parse_response {
        return 0;
    }

    let stream = this.response_stream;
    this.parse_stream(stream);
    0
}

pub struct HTTPDissector {
    url: String,
    parse_request: bool,
    request_content_type: String,
    request_header: String,
    request_headers: HashMap<String, String>,
    request_stream: *mut gmime_sys::GMimeStream,
    parse_response: bool,
    response_content_type: String,
    response_header: String,
    response_headers: HashMap<String, String>,
    request_parser: *const Parser,
    response_parser: *const Parser,
    response_stream: *mut gmime_sys::GMimeStream,
}

impl HTTPDissector {
    pub fn new(detector: Rc<Detector>, flow: *const c_char) -> Rc<RefCell<TCPDissector>> {
        let url = detector.get_http_url(flow);
        trace!("url = {}", url);
        let http = Rc::new(RefCell::new(HTTPDissector {
            url,
            parse_request: true,
            request_content_type: String::new(),
            request_header: String::new(),
            request_headers: HashMap::new(),
            request_stream: ptr::null_mut() as *mut gmime_sys::GMimeStream,
            parse_response: true,
            response_content_type: String::new(),
            response_header: String::new(),
            response_headers: HashMap::new(),
            request_parser: ptr::null(),
            response_parser: ptr::null(),
            response_stream: ptr::null_mut() as *mut gmime_sys::GMimeStream,
        }));

        let this = http.as_ptr() as *const c_char;

        unsafe {
            let request_parser = malloc(mem::size_of::<Parser>()) as *mut Parser;
            http_parser_init(request_parser, HttpParserType::Request);
            (*request_parser).data = this;

            let response_parser = malloc(mem::size_of::<Parser>()) as *mut Parser;
            http_parser_init(response_parser, HttpParserType::Response);
            (*response_parser).data = this;

            http.borrow_mut().request_parser = request_parser;
            http.borrow_mut().response_parser = response_parser;
        }
        return http;
    }

    fn parse_stream(&mut self, stream: *mut gmime_sys::GMimeStream) {
        unsafe {
            gmime_sys::g_mime_stream_seek(stream, 0, 0);
        }
        let mut parser = MimeParser::new(stream);
        let cb = Box::new(
            move |data: &[u8], is_test: bool, filename: String, mime_type: String| {
                if is_test {
                    debug!("text {}", mime_type);
                    return;
                }

                let result = mime::magic_buffer(data);
                match result {
                    Some(type_str) => {
                        debug!("buffer type = {}", type_str);
                    }
                    None => {
                        debug!(" buffer not find {}", mime_type);
                    }
                }

                let result = mime::find_magic_type(&mime_type);
                match result {
                    Some(type_str) => {
                        debug!("mime type = {}", type_str);
                    }
                    None => {
                        debug!("mime not find {}", mime_type);
                    }
                }
            },
        );
        let result = parser.parse(cb);
        match result {
            Err(_) => {
                debug!("mime parse error ");
            }

            Ok(_) => {}
        }
    }
}

impl Drop for HTTPDissector {
    fn drop(&mut self) {
        unsafe {
            free(self.request_parser as *mut c_void);
            free(self.response_parser as *mut c_void);

            if self.request_stream != ptr::null_mut() {
                gobject_2_0_sys::g_object_unref(self.request_stream as *mut c_void);
            }

            if self.response_stream != ptr::null_mut() {
                gobject_2_0_sys::g_object_unref(self.response_stream as *mut c_void);
            }
        }
    }
}

impl TCPDissector for HTTPDissector {
    fn on_client_data(&mut self, data: &[u8]) -> Result<(), ()> {
        unsafe {
            let n = http_parser_execute(
                self.request_parser,
                &REQUEST_SETTING as *const ParserSettings,
                data.as_ptr() as *const c_char,
                data.len() as isize,
            );

            if n != data.len() as isize {
                let c_str =
                    CStr::from_ptr(http_errno_description_from_parser(self.response_parser));
                let s = c_str.to_string_lossy();

                trace!("http parse error {}", s);
                Err(())
            } else {
                Ok(())
            }
        }
    }
    fn on_server_data(&mut self, data: &[u8]) -> Result<(), ()> {
        unsafe {
            let n = http_parser_execute(
                self.response_parser,
                &RESPONSE_SETTING as *const ParserSettings,
                data.as_ptr() as *const c_char,
                data.len() as isize,
            );

            if n != data.len() as isize {
                let err = http_errno_description_from_parser(self.response_parser);
                let c_str = CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(
                    err as *const u8,
                    strlen(err),
                ));
                let s = c_str.to_string_lossy();

                trace!("http parse error {}", s);
                Err(())
            } else {
                Ok(())
            }
        }
    }
}
