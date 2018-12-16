use crate::inet;
use crate::packet::Packet;
use libc::c_char;
use std::sync::Arc;
use std::ptr;
use std::slice;

#[link(name = "layerscpp")]
extern "C" {
    fn new_tcp_data_tracker(_seq: u32) -> *const c_char;
    fn tcp_data_tracker_set_callback(_tracker: *const c_char, flow: *const TcpFlow, _cb: extern "C" fn(*const TcpFlow, *const c_char, u32));
    fn tcp_data_tracker_process_data(_tracker: *const c_char, _data: *const c_char, _len: u32);
    fn free_tcp_data_tracker(_tracker: *const c_char);
}

type DataCallback = Fn(&[u8]);

#[repr(C)]
pub struct TcpFlow {
    on_data_callback: Box<DataCallback>,
    tracker_: *const c_char,
}


extern "C" fn on_data_callback(flow: *const TcpFlow, data: *const c_char, len: u32) {
    debug!("on data call back {}", len);
    unsafe {
        let payload = slice::from_raw_parts(data as *const u8, len as usize);
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

            {
                let abc = &*flow;
                tcp_data_tracker_set_callback(flow.tracker_, abc, on_data_callback);
            }

            return flow;
        }
    }

    pub fn process_packet(&mut self, packet: &Arc<Packet>) {
        let payload = packet.tcp_payload();
        if payload.len() == 0 {
            return;
        }
        unsafe {
            tcp_data_tracker_process_data(
                self.tracker_,
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

