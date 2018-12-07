use crate::config;
use crate::layer::tcp::TCPTracker;
use crate::packet::Packet;
use std::num::Wrapping;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Dispatcher {
    n_threads: u8,
    threads: Vec<thread::JoinHandle<()>>,
    senders: Vec<mpsc::Sender<Arc<Packet>>>,
}

impl Dispatcher {
    pub fn dispatch(&self, packet: Arc<Packet>) {
        let hash = (Wrapping(packet.src_ip)
            + Wrapping(packet.src_port as u32)
            + Wrapping(packet.dst_ip)
            + Wrapping(packet.dst_port as u32))
            % Wrapping(self.n_threads as u32);
        self.senders[hash.0 as usize]
            .send(packet)
            .expect("channel send error");
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

        let config = conf.clone();

        let cb = move || {
            let mut tcp_tracker = Box::new(TCPTracker::new(config));

            let timeout = Duration::new(10, 0);

            loop {
                match rx.recv_timeout(timeout) {
                    Ok(packet) => {
                        if packet.flag & Packet::TCP > 0 {
                            trace!(
                                "{}:{} ->{}:{}",
                                packet.src_ip_str(),
                                packet.src_port,
                                packet.dst_ip_str(),
                                packet.dst_port
                            );
                            TCPTracker::on_packet(&mut tcp_tracker, &packet)
                        }
                    }
                    Err(e) => match e {
                        mpsc::RecvTimeoutError::Timeout => {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                                * 1000
                                * 1000;
                            tcp_tracker.cleanup_stream(now);
                        }

                        mpsc::RecvTimeoutError::Disconnected => {
                            debug!("Disconnected");
                            return;
                        }
                    },
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
