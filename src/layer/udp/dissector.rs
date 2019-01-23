use layer::packet::Packet;
use std::sync::Arc;

pub trait UDPDissector {
    fn on_client_packet(&mut self, packet: &Arc<Packet>) -> Result<(), ()>;
    fn on_server_packet(&mut self, packet: &Arc<Packet>) -> Result<(), ()>;
}
