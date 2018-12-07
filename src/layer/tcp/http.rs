use crate::detector::Detector;
use crate::layer::TCPDissector;
use libc::c_char;
use std::cell::RefCell;
use std::rc::Rc;

pub struct HTTPDissector {
    detector: Rc<Detector>,
    flow: *const c_char,
}

impl HTTPDissector {
    pub fn new(detector: Rc<Detector>, flow: *const c_char) -> Rc<RefCell<TCPDissector>> {
        let http = HTTPDissector { detector, flow };

        debug!("http request {}", http.detector.get_http_url(http.flow));

        Rc::new(RefCell::new(http))
    }
}

impl TCPDissector for HTTPDissector {
    fn on_client_data(&mut self, data: &[u8]) {
        debug!("http client data {}", data.len());
    }
    fn on_server_data(&mut self, data: &[u8]) {
        debug!("http server data {}", data.len());
    }
}
