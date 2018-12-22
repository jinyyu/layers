use crate::detector::Detector;
use crate::layer::TCPDissector;
use libc::{c_char, c_void, free, malloc};
use std::cell::RefCell;
use std::collections::HashMap;
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
    Response,
    Both,
}

#[repr(C)]
struct Parser {
    opaque1: u32,
    nread: u32,
    content_length: u64,

    http_major: u16,
    http_minor: u16,
    status: u16,
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
}

extern "C" fn on_chunk_header(_parser: *const Parser) -> i32 {
    0
}

extern "C" fn on_chunk_complete(_parser: *const Parser) -> i32 {
    0
}

extern "C" fn on_request_message_begin(_parser: *const Parser) -> i32 {
    0
}

extern "C" fn on_url(_parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    let s = unsafe { String::from_raw_parts(data as *mut u8, length as usize, length as usize) };
    debug!("url = {}", s);
    0
}

extern "C" fn on_request_header_field(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    unsafe {
        let s = String::from_raw_parts(data as *mut u8, length as usize, length as usize);
        let this = (*parser).data as *mut HTTPDissector;
        (*this).request_header = s;
    }
    0
}

extern "C" fn on_request_header_value(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    unsafe {
        let v = String::from_raw_parts(data as *mut u8, length as usize, length as usize);
        let this = (*parser).data as *mut HTTPDissector;

        let k = (*this).request_header.clone();

        (*this).request_headers.insert(k, v);
    }
    0
}

extern "C" fn on_request_headers_complete(parser: *const Parser) -> i32 {
    unsafe {
        let this = (*parser).data as *const HTTPDissector;
        for (k, v) in &(*this).request_headers {
            trace!("{}: {}", k, v);
        }
    }
    0
}

extern "C" fn on_request_body(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    trace!("on_request_body");
    0
}

extern "C" fn on_request_message_complete(parser: *const Parser) -> i32 {
    trace!("on_request_message_complete");
    0
}

extern "C" fn on_response_message_begin(parser: *const Parser) -> i32 {
    trace!("on_response_message_begin");
    0
}

extern "C" fn on_status(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    unsafe {
        if (*parser).status != 200 {
            let s = String::from_raw_parts(data as *mut u8, length as usize, length as usize);
            trace!("http error : {} {}", (*parser).status, s);
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
    unsafe {
        let s = String::from_raw_parts(data as *mut u8, length as usize, length as usize);
        let this = (*parser).data as *mut HTTPDissector;
        (*this).response_header = s;
    }
    0
}

extern "C" fn on_response_header_value(
    parser: *const Parser,
    data: *const c_char,
    length: isize,
) -> i32 {
    unsafe {
        let v = String::from_raw_parts(data as *mut u8, length as usize, length as usize);
        let this = (*parser).data as *mut HTTPDissector;

        let k = (*this).response_header.clone();

        (*this).response_headers.insert(k, v);
    }
    0
}

extern "C" fn on_response_headers_complete(parser: *const Parser) -> i32 {
    trace!("on_response_headers_complete");
    0
}

extern "C" fn on_response_body(parser: *const Parser, data: *const c_char, length: isize) -> i32 {
    trace!("on_response_body");
    0
}

extern "C" fn on_response_message_complete(parser: *const Parser) -> i32 {
    trace!("on_response_message_complete");
    0
}

pub struct HTTPDissector {
    url: String,
    request_header: String,
    request_headers: HashMap<String, String>,
    response_header: String,
    response_headers: HashMap<String, String>,
    flow: *const c_char,
    buffer: Vec<u8>,
    request_parser: *const Parser,
    response_parser: *const Parser,
}

impl HTTPDissector {
    pub fn new(detector: Rc<Detector>, flow: *const c_char) -> Rc<RefCell<TCPDissector>> {
        let url = detector.get_http_url(flow);
        trace!("url = {}", url);
        let http = Rc::new(RefCell::new(HTTPDissector {
            url,
            request_header: "".to_string(),
            request_headers: HashMap::new(),
            response_header: "".to_string(),
            response_headers: HashMap::new(),
            flow,
            buffer: Vec::new(),
            request_parser: ptr::null(),
            response_parser: ptr::null(),
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
}

impl Drop for HTTPDissector {
    fn drop(&mut self) {
        unsafe {
            free(self.request_parser as *mut c_void);
            free(self.response_parser as *mut c_void);
        }
        let mut file = File::create("/tmp/foo.txt").unwrap();
        let result = file.write(self.buffer.as_slice());
        result.unwrap();
    }
}

impl TCPDissector for HTTPDissector {
    fn on_client_data(&mut self, data: &[u8]) -> Result<(), ()> {
        let n = unsafe {
            http_parser_execute(
                self.request_parser,
                &REQUEST_SETTING as *const ParserSettings,
                data.as_ptr() as *const c_char,
                data.len() as isize,
            )
        };
        if n != data.len() as isize {
            trace!("http parse error");
            Err(())
        } else {
            Ok(())
        }
    }
    fn on_server_data(&mut self, data: &[u8]) -> Result<(), ()> {
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
            trace!("http parse error");
            Err(())
        } else {
            Ok(())
        }
    }
}
