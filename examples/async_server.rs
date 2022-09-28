use bitcask::AsyncServer;
use bitcask::Engine;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let path: PathBuf = "./testdata".into();

    let server = AsyncServer::new(Engine::SLED, Some(path)).unwrap();

    server.run("0.0.0.0:9000").await.unwrap();
}
