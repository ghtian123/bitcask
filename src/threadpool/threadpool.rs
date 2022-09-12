use super::ThreadPool;
use crossbeam::channel::{self, Receiver, Sender};
use std::thread;

pub struct NaiveThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
}

impl ThreadPool for NaiveThreadPool {
    fn new(size: usize) -> Self {
        let (rx, rc) = channel::unbounded::<Message>();

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            workers.push(Worker::new(i, rc.clone()));
        }

        Self {
            workers: workers,
            sender: rx,
        }
    }

    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Message::Job(Box::new(f));

        self.sender.send(job).unwrap();
    }
}

impl Drop for NaiveThreadPool {
    fn drop(&mut self) {
        println!("send termintae to  all worker");

        for _ in &self.workers {
            self.sender.send(Message::Close).unwrap();
        }

        //等待任务结束
        for worker in &mut self.workers {
            // 获得所有权
            if let Some(t) = worker.handler.take() {
                t.join().unwrap();
            }
            // worker.handler.join().unwrap();
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    Close,
    Job(Job),
}

struct Worker {
    id: usize,
    handler: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, rc: Receiver<Message>) -> Self {
        let t = thread::spawn(move || loop {
            let messages = rc.recv().unwrap();
            match messages {
                Message::Close => {
                    println!("close worker {}", id);
                    return;
                }

                Message::Job(job) => {
                    println!("worker execute job id {}", id);
                    // catch_unwind(||job());
                    job()
                }
            }
        });

        Self {
            id: id,
            handler: Some(t),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if thread::panicking() {
            println!("thread panic id {}", self.id);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {
        let p = NaiveThreadPool::new(2);

        p.spawn(|| println!("hello world!"));

        // p.execute(|| panic!("zz"));

        p.spawn(|| println!("hello world!"));
        p.spawn(|| println!("hello world!"));

        for i in 0..10 {
            thread::sleep(std::time::Duration::from_secs(1));
            p.spawn(move || println!("hello world! {}", i));
        }

        // p.execute(|| 2);

        thread::sleep(std::time::Duration::from_secs(20));
    }
}
