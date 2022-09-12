use crate::common::{request, response};
use crate::new_engine;
use crate::Engine;
use crate::KvError;
use crate::KvsEngine;
use crate::ThreadPool;
use serde_json::Deserializer;
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct KvServer<TP: ThreadPool> {
    engine: Arc<Mutex<Box<dyn KvsEngine>>>,
    pool: TP,
}

impl<TP: ThreadPool> KvServer<TP> {
    pub fn new<P>(e: Engine, path: Option<P>) -> Result<Self, KvError>
    where
        P: Into<PathBuf>,
    {
        let engine = new_engine(e, path)?;

        let pool = ThreadPool::new(2);

        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
            pool: pool,
        })
    }

    pub fn run<A: ToSocketAddrs>(self, addr: A) -> Result<(), KvError> {
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            let engine = self.engine.clone();

            self.pool.spawn(move || match stream {
                Ok(stream) => {
                    if let Err(e) = serve(engine, stream) {
                        println!("send err ->{:?}", e);
                    }
                }
                Err(e) => println!("stream err {:?}", e),
            })
        }

        Ok(())
    }
}

fn serve(engine: Arc<Mutex<Box<dyn KvsEngine>>>, stream: TcpStream) -> Result<(), KvError> {
    let peer_addr = stream.peer_addr()?;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let req_reader = Deserializer::from_reader(reader).into_iter::<request>();

    for req in req_reader {
        let req = req?;
        let mut resp: response;
        match req {
            request::Get { key } => {
                let mut e = engine.lock().unwrap();

                let v = e.get(key);

                match v {
                    Ok(key) => {
                        resp = response::Ok(key);
                    }

                    Err(e) => {
                        resp = response::Err(e.to_string());
                    }
                }
            }
            request::Set { key, value } => {
                let mut e = engine.lock().unwrap();

                let v = e.set(key, value);

                match v {
                    Ok(key) => {
                        resp = response::Ok(None);
                    }

                    Err(e) => {
                        resp = response::Err(e.to_string());
                    }
                }
            }
            request::Remove { key } => {
                let mut e = engine.lock().unwrap();

                let v = e.remove(key);

                match v {
                    Ok(key) => {
                        resp = response::Ok(None);
                    }

                    Err(e) => {
                        resp = response::Err(e.to_string());
                    }
                }
            }
        }

        serde_json::to_writer(&mut writer, &resp)?;
        writer.flush()?;
        println!("Response sent to {}: {:?}", peer_addr, resp);
    }
    Ok(())
}
