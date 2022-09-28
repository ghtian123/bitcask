use crate::KvError;
use std::path::PathBuf;
mod kv;
mod sled;

pub trait KvsEngine: Send {
    fn set(&mut self, key: String, value: String) -> Result<(), KvError>;

    fn get(&mut self, key: String) -> Result<Option<String>, KvError>;

    fn remove(&mut self, key: String) -> Result<(), KvError>;
}

pub enum Engine {
    BITCASK,
    SLED,
}

pub fn new_engine<P>(e: Engine, path: Option<P>) -> Result<Box<dyn KvsEngine>, KvError>
where
    P: Into<PathBuf>,
{
    match e {
        Engine::BITCASK => {
            let k = kv::KvStore::open(path.unwrap())?;
            Ok(Box::new(k))
        }
        Engine::SLED => {
            let k = sled::SledKvsEngine::open(path.unwrap())?;
            Ok(Box::new(k))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {}
}
