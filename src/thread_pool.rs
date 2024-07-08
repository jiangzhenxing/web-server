use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Receiver, Sender};

pub struct ThreadPool {
    sender: Option<Sender<Job>>,
    reciever: Arc<Mutex<Receiver<Job>>>,
    workers: Vec<Worker>
}

impl ThreadPool {
    /// 创建新线程池
    /// 
    /// num：线程池中线程数量
    /// 
    /// # Panic
    /// 
    /// `new` 函数在 size 为 0 时会 panic。
    pub fn new(num: usize) -> ThreadPool {
        assert!(num > 0);
        let (sender, reciever) = mpsc::channel::<Job>();
        let reciever = Arc::new(Mutex::new(reciever));
        let workers: Vec<Worker> = (0..num)
            .map(|id| Worker::create(id, Arc::clone(&reciever)))
            .collect();
        ThreadPool {
            sender: Some(sender), reciever, workers
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(sender) = self.sender.as_ref() {
            let job = Job::from_task(Box::new(f));
            sender.clone().send(job).unwrap();
        }
    }

    pub fn shutdown(self) {
        drop(self);
        // for _ in 0..self.workers.len() {
        //     if let Some(sender) = self.sender.as_ref() {
        //         sender.clone().send(Job::stop_job()).unwrap();
        //     }
        // }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>
}

impl Worker {
    fn new(id: usize, thread: JoinHandle<()>) -> Worker {
        Worker {id, thread: Some(thread)}
    }
    fn create(id: usize, reciever: Arc<Mutex<Receiver<Job>>>) -> Worker {
        println!("create Worker {id}");
        let thread = thread::spawn(move|| {
            loop {
                let job = {
                    reciever.lock().unwrap().recv()
                };
                match job {
                    Ok(job) => {
                        println!("worker {id} recieve a job");
                        if job.stop {
                            break
                        }
                        job.start();
                    },
                    Err(_) => {
                        println!("worker {id} shutdown");
                        break;
                    }
                }
                
            }
        });
        Worker{id, thread: Some(thread)}
    }
}

struct Job {
    stop: bool,
    task: Box<dyn FnOnce() + Send + 'static>,
}

impl Job {
    fn new(stop: bool, task: Box<dyn FnOnce() + Send + 'static>) -> Job {
        Job {stop, task}
    }
    fn from_task(task: Box<dyn FnOnce() + Send + 'static>) -> Job {
        Self::new(false, task)
    }
    fn stop_job() -> Job {
        Self::new(true, Box::new(||{}))
    }
    fn start(self) {
        (self.task)();
    }
}