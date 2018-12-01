

pub trait Dissector {
    fn on_client_data(&mut self, data: &[u8]);
    fn on_server_data(&mut self, data: &[u8]);
}
