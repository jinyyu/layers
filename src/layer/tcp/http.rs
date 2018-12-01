use layer::TCPDissector;
use std::rc::Rc;
use std::cell::RefCell;


pub struct HTTPDissector {}

impl HTTPDissector {
    pub fn new() -> Rc<RefCell<TCPDissector>> {
        Rc::new(RefCell::new(HTTPDissector {}))
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
