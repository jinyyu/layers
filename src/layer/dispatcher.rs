use crate::layer::{TCPTracker, UDPTracker};
use layer::packet::Packet;
use std::num::Wrapping;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Barrier};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Dispatcher {
    running: Arc<AtomicBool>,
    barrier: Arc<Barrier>,
    n_threads: u8,
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

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        self.barrier.wait();
        debug!("app stopped")
    }

    fn worker(
        running: Arc<AtomicBool>,
        barrier: Arc<Barrier>,
        receiver: mpsc::Receiver<Arc<Packet>>,
    ) {
        let mut tcp_tracker = Box::new(TCPTracker::new());
        let mut udp_tracker = Box::new(UDPTracker::new());

        let timeout = Duration::new(1, 0);

        loop {
            if !running.load(Ordering::Relaxed) {
                debug!("stop running");
                barrier.wait();
                return;
            }
            match receiver.recv_timeout(timeout) {
                Ok(packet) => {
                    Dispatcher::dispatch_packet(&mut tcp_tracker, &mut udp_tracker, &packet);
                    continue;
                }
                Err(e) => match e {
                    mpsc::RecvTimeoutError::Timeout => {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                            * 1000
                            * 1000;
                        tcp_tracker.cleanup_stream(now) + udp_tracker.cleanup_stream(now);
                    }

                    mpsc::RecvTimeoutError::Disconnected => {
                        info!("Disconnected");
                        return;
                    }
                },
            }
        }
    }

    fn dispatch_packet(tcp: &mut Box<TCPTracker>, udp: &mut Box<UDPTracker>, packet: &Arc<Packet>) {
        if packet.state & Packet::STATE_TCP > 0 {
            trace!(
                "tcp {}:{} ->{}:{}",
                packet.src_ip_str(),
                packet.src_port,
                packet.dst_ip_str(),
                packet.dst_port
            );
            tcp.on_packet(packet);
            return;
        }

        if packet.state & Packet::STATE_UDP > 0 {
            trace!(
                "udp {}:{} ->{}:{}",
                packet.src_ip_str(),
                packet.src_port,
                packet.dst_ip_str(),
                packet.dst_port
            );
            udp.on_packet(packet);
            return;
        }
    }
}

pub fn init(n_threads: u8) -> Arc<Dispatcher> {
    let mut dispatcher = Dispatcher {
        running: Arc::new(AtomicBool::new(true)),
        barrier: Arc::new(Barrier::new((n_threads + 1) as usize)),
        n_threads,
        senders: Vec::new(),
    };

    for _i in 0..n_threads {
        let running = dispatcher.running.clone();
        let barrier = dispatcher.barrier.clone();
        let (tx, rx) = mpsc::channel::<Arc<Packet>>();

        let cb = move || Dispatcher::worker(running, barrier, rx);

        thread::spawn(cb);
        dispatcher.senders.push(tx);
    }

    debug!("threads = {}", dispatcher.n_threads);

    return Arc::new(dispatcher);
}
