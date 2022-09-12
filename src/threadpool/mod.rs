mod threadpool;
pub use threadpool::NaiveThreadPool;
use crate::KvError;

pub trait ThreadPool {
    fn new(threads: usize) -> Self
    where
        Self: Sized;

    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;
}
