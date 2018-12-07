use crate::inet;
use crate::packet::Packet;
use std::sync::Arc;

// As defined by RFC 1982 - 2 ^ (SERIAL_BITS - 1)
const SEQ_NUMBER_DIFF: u32 = 2147483648;

fn seq_compare(seq1: u32, seq2: u32) -> i8 {
    if seq1 == seq2 {
        return 0;
    }
    if seq1 < seq2 {
        if seq2 - seq1 < SEQ_NUMBER_DIFF {
            return -1;
        } else {
            return 1;
        }
    } else {
        if seq1 - seq2 > SEQ_NUMBER_DIFF {
            return -1;
        } else {
            return 1;
        }
    }
}

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
