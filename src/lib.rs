use std::sync::{Arc, mpsc, Mutex};
use std::thread;

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

struct ThreadWorker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl ThreadWorker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> ThreadWorker {
        ThreadWorker {
            id,
            thread: thread::spawn(move || {
                loop {
                    //we are using loop instead of while inorder to release lock quickly
                    let job = receiver.lock().unwrap().recv().unwrap();
                    job.call_box();
                }
            }),
        }
    }
}

pub struct ThreadPool {
    threads: Vec<ThreadWorker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (tx, rx) = mpsc::channel();

        //we are using Arc for multiple reference operation atomically and mutex for interior mutability
        let thread_safe_tx = Arc::new(Mutex::new(rx));

        //we are using Vec::with_capacity for efficiency as we know the capacity before hand
        let mut threads = Vec::with_capacity(size);
        for id in 0..size {
            threads.push(ThreadWorker::new(id, Arc::clone(&thread_safe_tx)));
        }
        ThreadPool { threads, sender: tx }
    }

    pub fn execute<T>(&self, f: T)
        where T: FnOnce() + Send + 'static {
        self.sender.send(Box::new(f)).unwrap();
    }
}