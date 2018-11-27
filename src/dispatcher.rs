use config;
use std::sync::Arc;
use std::thread;
use std::vec;
use std::sync::mpsc;
use packet::Packet;
use std::num::Wrapping;
use layer::*;

pub struct Dispatcher {
    n_threads: u8,
    threads: Vec<thread::JoinHandle<()>>,
    senders: Vec<mpsc::Sender<Arc<Packet>>>,
}

impl Dispatcher {
    pub fn dispatch(&self, packet: Arc<Packet>) {
        let hash = (Wrapping(packet.src_ip) + Wrapping(packet.src_port as u32) + Wrapping(packet.dst_ip) + Wrapping(packet.dst_port as u32))
            % Wrapping(self.n_threads as u32);
        self.senders[hash.0 as usize].send(packet).expect("channel send error");
    }
}

pub fn init(conf: Arc<config::Configure>) -> Arc<Dispatcher> {
    let mut dispatcher = Dispatcher {
        n_threads: conf.worker_thread as u8,
        threads: Vec::new(),
        senders: Vec::new(),
    };

    for _i in 0..conf.worker_thread {
        let (tx, rx) = mpsc::channel::<Arc<Packet>>();

        let cb = move || {
            loop {
                let mut tcp_tracker = TCPTracker::new();

                let packet = rx.recv().expect("channel receive error");

                if packet.flag & Packet::TCP > 0 {
                    tcp_tracker.get_mut().on_packet(packet);
                }

            }
        };

        let handle = thread::spawn(cb);

        dispatcher.threads.push(handle);
        dispatcher.senders.push(tx);
    }

    debug!("threads = {}", dispatcher.threads.len());

    return Arc::new(dispatcher);
}