use super::KvsEngine;
use crate::KvError;
use sled::{Db, Tree};
use std::path::PathBuf;

#[derive(Clone)]
pub struct SledKvsEngine(Db);

impl SledKvsEngine {
    pub fn open<T>(path: T) -> Result<SledKvsEngine, KvError>
    where
        T: Into<PathBuf>,
    {
        let db = sled::open(path.into())?;
        Ok(SledKvsEngine(db))
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<(), KvError> {
        let tree: &Tree = &self.0;
        tree.insert(key, value.into_bytes()).map(|_| ())?;
        tree.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>, KvError> {
        let tree: &Tree = &self.0;
        Ok(tree
            .get(key)?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }

    fn remove(&mut self, key: String) -> Result<(), KvError> {
        let tree: &Tree = &self.0;
        tree.remove(key)?.ok_or(KvError::KeyNotFound)?;
        tree.flush()?;
        Ok(())
    }
}
