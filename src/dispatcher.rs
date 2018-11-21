use config;
use std::sync::Arc;
use std::thread;
use std::vec;
use std::sync::mpsc;
use packet::Packet;

pub struct Dispatcher {
    n_threads: u8,
    threads: Vec<thread::JoinHandle<()>>,
    senders: Vec<mpsc::Sender<Arc<Packet>>>,
}

impl Dispatcher {
    pub fn dispatch(&self, packet: Arc<Packet>) {
        debug!("{}:{} ->{}:{}", packet.src_ip_str(), packet.src_port(), packet.dst_ip_str(), packet.dst_port());
        let hash = (packet.src_ip() + packet.src_port() as u32 + packet.dst_ip() + packet.dst_port() as u32) % self.n_threads as u32;
        self.senders[hash as usize].send(packet).expect("send error----------------------------------------------------------");
    }
}

pub fn init(conf: Arc<config::Configure>) -> Arc<Dispatcher> {
    let mut dispatcher = Dispatcher {
        n_threads: conf.worker_thread as u8,
        threads: Vec::new(),
        senders: Vec::new(),
    };

    for i in 0..conf.worker_thread {
        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            for received in rx {
                println!("recv packet");
            }
        });

        dispatcher.threads.push(handle);
        dispatcher.senders.push(tx);
    }


    debug!("threads = {}", dispatcher.threads.len());

    return Arc::new(dispatcher);
}