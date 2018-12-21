use crate::detector::Detector;
use crate::layer::TCPDissector;
use libc::{c_char, free, malloc};
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::vec;

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
    Resonse,
    Both,
}

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
}

extern "C" fn on_chunk_header(parser: *const Parser) -> i32 {
    debug!("on_chunk_header begin");
    0
}

extern "C" fn on_chunk_complete(parser: *const Parser) -> i32 {
    debug!("on_chunk_complete");
    0
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
    request_parser: *const Parser,
    response_parser: *const Parser,
}

impl HTTPDissector {
    pub fn new(detector: Rc<Detector>, flow: *const c_char) -> Rc<RefCell<TCPDissector>> {
        let http = Rc::new(RefCell::new(HTTPDissector {
            detector,
            flow,
            buffer: Vec::new(),
            request_parser: ptr::null(),
            response_parser: ptr::null(),
        }));

        let this = http.as_ptr() as *const c_char;

        unsafe {
            let mut request_parser = malloc(mem::size_of::<Parser>()) as *mut Parser;
            http_parser_init(request_parser, HttpParserType::Request);
            (*request_parser).data = this;

            let mut response_parser = malloc(mem::size_of::<Parser>()) as *mut Parser;
            http_parser_init(response_parser, HttpParserType::Resonse);
            (*response_parser).data = this;

            http.borrow_mut().request_parser = request_parser;
            http.borrow_mut().response_parser = response_parser;
        }
        debug!("http request {}", http.borrow().detector.get_http_url(flow));
        return http;
    }
}

impl Drop for HTTPDissector {
    fn drop(&mut self) {
        let mut file = File::create("/tmp/foo.txt").unwrap();
        let result = file.write(self.buffer.as_slice());
        result.unwrap();
    }
}

impl TCPDissector for HTTPDissector {
    fn on_client_data(&mut self, data: &[u8]) {
        let n = unsafe {
            http_parser_execute(
                self.request_parser,
                &REQUEST_SETTING as *const ParserSettings,
                data.as_ptr() as *const c_char,
                data.len() as isize,
            )
        };
        if n != data.len() as isize {
            debug!("http parse error");
        }
    }
    fn on_server_data(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);

        let n = unsafe {
            http_parser_execute(
                self.response_parser,
                &RESPONSE_SETTING as *const ParserSettings,
                data.as_ptr() as *const c_char,
                data.len() as isize,
            )
        };
        if n != data.len() as isize {
            debug!("http parse error");
        }
    }
}
