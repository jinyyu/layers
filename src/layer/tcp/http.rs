use crate::detector::Detector;
use crate::layer::TCPDissector;
use libc::c_char;
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::vec;

#[repr(C)]
struct Parser {
    opaque1: u32,
    nread: u32,
    content_length: u64,

    http_major: u16,
    http_minor: u16,
    opaque2: u32,

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
    /* When on_chunk_header is called, the current chunk length is stored
     * in parser->content_length.
     */
    on_chunk_header: HTTPCallback,
    on_chunk_complete: HTTPCallback,
}

extern "C" {
    fn init_http_parser_setting(_req: ParserSettings, _res: ParserSettings);
    fn new_http_parser(_ctx: *const c_char) -> *const c_char;
    fn free_http_parser(_parser: *const c_char);
    fn http_parser_execute_request(
        _parser: *const c_char,
        _data: *const c_char,
        len: isize,
    ) -> isize;
    fn http_parser_execute_response(
        _parser: *const c_char,
        _data: *const c_char,
        len: isize,
    ) -> isize;
}

extern "C" fn on_request_message_begin(parser: *const Parser) -> i32 {
    debug!("on_request_message_begin begin");
    0
}

extern "C" fn on_url(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    debug!("on_url");
    0
}

extern "C" fn on_request_header_field(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    debug!("on_request_header_field");
    0
}

extern "C" fn on_request_header_value(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    debug!("on_request_header_value");
    0
}

extern "C" fn on_request_headers_complete(parser: *const Parser) -> i32 {
    debug!("on_request_headers_complete");
    0
}

extern "C" fn on_request_body(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    debug!("on_request_body");
    0
}

extern "C" fn on_request_message_complete(parser: *const Parser) -> i32 {
    debug!("on_request_message_complete");
    0
}

extern "C" fn on_response_message_begin(parser: *const Parser) -> i32 {
    debug!("on_response_message_begin");
    0
}

extern "C" fn on_status(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    debug!("on_status");
    0
}

extern "C" fn on_response_header_field(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    debug!("on_response_header_field");
    0
}

extern "C" fn on_response_header_value(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    debug!("on_response_header_value");
    0
}

extern "C" fn on_response_headers_complete(parser: *const Parser) -> i32 {
    debug!("on_response_headers_complete");
    0
}

extern "C" fn on_response_body(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    debug!("on_response_body");
    0
}

extern "C" fn on_response_message_complete(parser: *const Parser) -> i32 {
    debug!("on_response_message_complete");
    0
}

impl Parser {
    fn new(data: *const c_char) -> Box<Parser> {
        Box::new(Parser {
            opaque1: 0,
            nread: 0,
            content_length: 0,
            http_major: 0,
            http_minor: 0,
            opaque2: 0,
            data,
        })
    }
}

pub struct HTTPDissector {
    detector: Rc<Detector>,
    flow: *const c_char,
    buffer: Vec<u8>,
    parser: *const c_char,
}

impl HTTPDissector {
    pub fn init() {
        let request_setting = ParserSettings {
            on_message_begin: on_request_message_begin,
            on_url,
            on_status: unsafe { mem::transmute::<*const i8, HTTPDataCallback>(ptr::null()) },
            on_header_field: on_request_header_field,
            on_header_value: on_request_header_value,
            on_headers_complete: on_request_headers_complete,
            on_body: on_request_body,
            on_message_complete: on_request_message_complete,
            on_chunk_header: unsafe { mem::transmute::<*const i8, HTTPCallback>(ptr::null()) },
            on_chunk_complete: unsafe { mem::transmute::<*const i8, HTTPCallback>(ptr::null()) },
        };

        let response_setting = ParserSettings {
            on_message_begin: on_response_message_begin,
            on_url: unsafe { mem::transmute::<*const i8, HTTPDataCallback>(ptr::null()) },
            on_status,
            on_header_field: on_response_header_field,
            on_header_value: on_response_header_value,
            on_headers_complete: on_response_headers_complete,
            on_body: on_response_body,
            on_message_complete: on_response_message_complete,
            on_chunk_header: unsafe { mem::transmute::<*const i8, HTTPCallback>(ptr::null()) },
            on_chunk_complete: unsafe { mem::transmute::<*const i8, HTTPCallback>(ptr::null()) },
        };

        unsafe {
            init_http_parser_setting(request_setting, response_setting);
        }
    }
    pub fn new(detector: Rc<Detector>, flow: *const c_char) -> Rc<RefCell<TCPDissector>> {
        let http = Rc::new(RefCell::new(HTTPDissector {
            detector,
            flow,
            buffer: Vec::new(),
            parser: ptr::null(),
        }));
        let raw = (*http).as_ptr();
        http.borrow_mut().parser = unsafe { new_http_parser(raw as *const c_char) };

        debug!("http request {}", http.borrow().detector.get_http_url(flow));
        return http;
    }
}

impl Drop for HTTPDissector {
    fn drop(&mut self) {
        unsafe { free_http_parser(self.parser) };

        let mut file = File::create("/tmp/foo.txt").unwrap();
        let result = file.write(self.buffer.as_slice());
        result.unwrap();
    }
}

impl TCPDissector for HTTPDissector {
    fn on_client_data(&mut self, data: &[u8]) {
        unsafe {
            http_parser_execute_request(
                self.parser,
                data.as_ptr() as *const c_char,
                data.len() as isize,
            );
        }
        debug!("http client data {}", data.len());
    }
    fn on_server_data(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);

        debug!("http server data {}", data.len());
        unsafe {
            http_parser_execute_response(
                self.parser,
                data.as_ptr() as *const c_char,
                data.len() as isize,
            );
        }
    }
}
