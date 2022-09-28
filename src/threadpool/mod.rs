mod threadpool;
use crate::KvError;
pub use threadpool::NaiveThreadPool;

pub trait ThreadPool {
    fn new(threads: usize) -> Self
    where
        Self: Sized;

    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;
}
