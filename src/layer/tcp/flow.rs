use inet;
use packet::Packet;
use std::sync::Arc;


type DataCallback = Fn(&[u8]);

pub struct TcpFlow {
    next_seq: u32,
    on_data: Box<DataCallback>,
}

impl TcpFlow {
    pub fn new(packet: &Arc<Packet>, on_data: Box<DataCallback>) -> TcpFlow {
        TcpFlow {
            next_seq: unsafe { inet::ntohl((*packet.tcp).seq) + 1 },
            on_data,
        }
    }

    pub fn handle_packet(&mut self, packet: &Arc<Packet>) {
        let payload = packet.tcp_payload();
        if payload.len() == 0 {
            return;
        }
        let seq = unsafe { inet::ntohl((*packet.tcp).seq) };

        if seq == self.next_seq {
            (*self.on_data)(payload);
            self.next_seq = seq + payload.len() as u32;
        } else {

        }
    }
}
