use crate::common::{request, response};
use crate::KvError;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub struct AsyncKvsClient {
    stream: Framed<TcpStream, LengthDelimitedCodec>,
}

impl AsyncKvsClient {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, KvError> {
        let stream = TcpStream::connect(addr).await?;
        let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
        Ok(Self { stream: stream })
    }

    pub async fn get(&mut self, key: String) -> Result<Option<String>, KvError> {
        self.send_request(request::Get { key: key }).await.unwrap();
        Ok(None)
    }

    async fn send_request(&mut self, req: request) -> Result<Option<response>, KvError> {
        println!("start to send req {:?}", req);
        self.stream
            .send(Bytes::from(serde_json::to_vec(&req)?))
            .await?;

        if let Some(Ok(data)) = self.stream.next().await {
            let res_reader =
                serde_json::Deserializer::from_slice(data.as_ref()).into_iter::<response>();
            for res in res_reader {
                if let Ok(r) = res {
                    println!("GOT--> {:?}", r);
                }
            }
        }
        Ok(None)
    }

    pub async fn set(&mut self, key: String, value: String) -> Result<(), KvError> {
        self.send_request(request::Set {
            key: key,
            value: value,
        })
        .await
        .unwrap();
        Ok(())
    }

    pub async fn remove(&mut self, key: String) -> Result<(), KvError> {
        self.send_request(request::Remove { key: key })
            .await
            .unwrap();

        Ok(())
    }
}
