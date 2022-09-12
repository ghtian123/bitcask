use crate::common::{request, response};
use crate::KvError;
use serde::Deserialize;
use serde_json::de::{Deserializer, IoRead};
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpStream, ToSocketAddrs};

/// Key value store client
pub struct KvsClient {
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>,
}

impl KvsClient {
    /// Connect to `addr` to access `KvsServer`.
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, KvError> {
        let tcp_reader = TcpStream::connect(addr)?;
        let tcp_writer = tcp_reader.try_clone()?;
        Ok(KvsClient {
            reader: Deserializer::from_reader(BufReader::new(tcp_reader)),
            writer: BufWriter::new(tcp_writer),
        })
    }

    /// Get the value of a given key from the server.
    pub fn get(&mut self, key: String) -> Result<Option<String>, KvError> {
        serde_json::to_writer(&mut self.writer, &request::Get { key })?;
        self.writer.flush()?;
        let resp = response::deserialize(&mut self.reader)?;
        match resp {
            response::Ok(res) => Ok(res),
            response::Err(e) => Err(KvError::StringError(e)),
        }
    }

    /// Set the value of a string key in the server.
    pub fn set(&mut self, key: String, value: String) -> Result<(), KvError> {
        serde_json::to_writer(&mut self.writer, &request::Set { key, value })?;
        self.writer.flush()?;
        let resp = response::deserialize(&mut self.reader)?;
        match resp {
            response::Ok(res) => Ok(()),
            response::Err(e) => Err(KvError::StringError(e)),
        }
    }

    /// Remove a string key in the server.
    pub fn remove(&mut self, key: String) -> Result<(), KvError> {
        serde_json::to_writer(&mut self.writer, &request::Remove { key })?;
        self.writer.flush()?;
        let resp = response::deserialize(&mut self.reader)?;
        match resp {
            response::Ok(_) => Ok(()),
            response::Err(e) => Err(KvError::StringError(e)),
        }
    }
}
