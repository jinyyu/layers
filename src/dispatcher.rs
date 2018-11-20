use config;
use std::rc::Rc;
use std::thread;
use std::vec;
use std::sync::mpsc;

pub struct Dispatcher {
    n_threads: u8,
    threads: Vec<thread::JoinHandle<()>>,
}

impl Dispatcher {
    pub fn dispatch(&self){

    }

}

pub fn init(conf: &config::Configure) -> Rc<Dispatcher> {
    let mut dispatcher = Dispatcher {
        n_threads: conf.worker_thread as u8,
        threads: Vec::new(),
    };

    for i in 0..dispatcher.n_threads {
        let (tx, rx) = mpsc::channel::<u8>();
        let handle = thread::spawn(|| {});
        dispatcher.threads.push(handle);
    }
    debug!("threads = {}", dispatcher.threads.len());

    return Rc::new(dispatcher);
}