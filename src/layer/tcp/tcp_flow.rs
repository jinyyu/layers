use crate::inet;
use crate::packet::Packet;
use layer::tcp::TCPHeader;
use libc::c_char;
use std::mem;
use std::slice;
use std::sync::Arc;

#[link(name = "layerscpp")]
extern "C" {
    fn new_tcp_data_tracker(_seq: u32) -> *const c_char;
    fn tcp_data_tracker_set_callback(
        _tracker: *const c_char,
        flow: *const c_char,
        _cb: extern "C" fn(*const c_char, *const c_char, u32),
    );

    fn tcp_data_tracker_update_seq(_tracker: *const c_char, _seq: u32);
    fn tcp_data_tracker_process_data(
        _tracker: *const c_char,
        _seq: u32,
        _data: *const c_char,
        _len: u32,
    );
    fn free_tcp_data_tracker(_tracker: *const c_char);
}

type DataCallback = Fn(&[u8]);

pub struct TcpFlow {
    on_data_callback: Box<DataCallback>,
    tracker_: *const c_char,
}

extern "C" fn on_data_callback(flow: *const c_char, data: *const c_char, len: u32) {
    unsafe {
        let payload = slice::from_raw_parts(data as *const u8, len as usize);
        let flow = mem::transmute::<*const c_char, *const TcpFlow>(flow);
        (*(*flow).on_data_callback)(payload);
    }
}

impl TcpFlow {
    pub fn new(packet: &Arc<Packet>, callback: Box<DataCallback>) -> Box<TcpFlow> {
        unsafe {
            let flow = Box::new(TcpFlow {
                on_data_callback: callback,
                tracker_: new_tcp_data_tracker(inet::ntohl((*packet.tcp).seq) + 1),
            });

            let this = mem::transmute::<*const TcpFlow, *const c_char>(&*flow);
            tcp_data_tracker_set_callback(flow.tracker_, this, on_data_callback);

            return flow;
        }
    }

    pub fn process_packet(&mut self, packet: &Arc<Packet>) {
        unsafe {
            if (*packet.tcp).flags & TCPHeader::SYN > 0 {
                let seq = inet::ntohl((*packet.tcp).seq);
                tcp_data_tracker_update_seq(self.tracker_, seq);
            }
        }

        let payload = packet.tcp_payload();
        if payload.len() == 0 {
            return;
        }
        unsafe {
            tcp_data_tracker_process_data(
                self.tracker_,
                inet::ntohl((*packet.tcp).seq),
                payload.as_ptr() as *const c_char,
                payload.len() as u32,
            );
        }
    }
}

impl Drop for TcpFlow {
    fn drop(&mut self) {
        unsafe {
            free_tcp_data_tracker(self.tracker_);
        }
    }
}
