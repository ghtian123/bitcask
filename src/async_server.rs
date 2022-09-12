use crate::common::{request, response};
use crate::new_engine;
use crate::Engine;
use crate::KvError;
use crate::KvsEngine;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub struct AsyncServer {
    engine: Arc<Mutex<Box<dyn KvsEngine>>>,
}

impl AsyncServer {
    pub fn new<P>(e: Engine, path: Option<P>) -> Result<Self, KvError>
    where
        P: Into<PathBuf>,
    {
        let engine = new_engine(e, path)?;

        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
        })
    }

    pub async fn run<A: ToSocketAddrs>(self, addr: A) -> Result<(), KvError> {
        let listener = TcpListener::bind(addr).await?;

        loop {
            println!("start to accept");
            let (mut stream, _) = listener.accept().await?;

            let engine = self.engine.clone();
            tokio::spawn(async move {
                if let Err(e) = serve(engine, stream).await {
                    println!("send err ->{:?}", e);
                }
            });
        }
    }
}

async fn serve(
    engine: Arc<Mutex<Box<dyn KvsEngine>>>,
    mut stream: TcpStream,
) -> Result<(), KvError> {
    let mut stream = Framed::new(stream, LengthDelimitedCodec::new());

    while let Some(Ok(data)) = stream.next().await {
        let req_reader = serde_json::Deserializer::from_slice(data.as_ref()).into_iter::<request>();

        for req in req_reader {
            let req = req?;
            let mut resp: response;
            match req {
                request::Get { key } => {
                    let mut e = engine.lock().unwrap();

                    match e.get(key) {
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

                    match e.set(key, value) {
                        Ok(_) => {
                            resp = response::Ok(None);
                        }

                        Err(e) => {
                            resp = response::Err(e.to_string());
                        }
                    }
                }
                request::Remove { key } => {
                    let mut e = engine.lock().unwrap();

                    match e.remove(key) {
                        Ok(_) => {
                            resp = response::Ok(None);
                        }

                        Err(e) => {
                            resp = response::Err(e.to_string());
                        }
                    }
                }
            }

            println!("start to send {:?}", resp);
            stream.send(Bytes::from(serde_json::to_vec(&resp)?)).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}
