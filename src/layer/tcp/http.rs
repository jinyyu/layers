use crate::detector::Detector;
use crate::layer::TCPDissector;
use libc::c_char;
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
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

extern "C" fn on_request_message_begin(parser: *const Parser) -> i32 {
    0
}

extern "C" fn on_url(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    0
}

extern "C" fn on_request_header_field(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    0
}

extern "C" fn on_request_header_value(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    0
}

extern "C" fn on_request_headers_complete(parser: *const Parser) -> i32 {
    0
}

extern "C" fn on_request_body(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    0
}

extern "C" fn on_request_message_complete(parser: *const Parser) -> i32 {
    0
}

extern "C" fn on_response_message_begin(parser: *const Parser) -> i32 {
    0
}

extern "C" fn on_status(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    0
}

extern "C" fn on_response_header_field(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    0
}

extern "C" fn on_response_header_value(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    0
}

extern "C" fn on_response_headers_complete(parser: *const Parser) -> i32 {
    0
}

extern "C" fn on_response_body(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    0
}

extern "C" fn on_response_message_complete(parser: *const Parser) -> i32 {
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
}

impl HTTPDissector {
    pub fn new(detector: Rc<Detector>, flow: *const c_char) -> Rc<RefCell<TCPDissector>> {
        let http = HTTPDissector {
            detector,
            flow,
            buffer: Vec::new(),
        };

        debug!("http request {}", http.detector.get_http_url(http.flow));

        Rc::new(RefCell::new(http))
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
        debug!("http client data {}", data.len());
    }
    fn on_server_data(&mut self, data: &[u8]) {
        //debug!("http server data {}", data.len());

        self.buffer.extend_from_slice(data)
    }
}
