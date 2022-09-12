pub mod engines;
mod error;
pub use engines::*;
pub use error::KvError;
mod threadpool;
pub use threadpool::{NaiveThreadPool, ThreadPool};
mod server;
pub use server::*;
mod client;
mod common;
pub use client::*;
mod async_client;
pub use async_client::*;
mod async_server;
pub use async_server::*;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
