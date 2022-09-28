use bitcask::Engine;
use bitcask::KvServer;
use bitcask::NaiveThreadPool;
use std::path::PathBuf;

fn main() {
    let path: PathBuf = "./testdata".into();

    let server: KvServer<NaiveThreadPool> = KvServer::new(Engine::SLED, Some(path)).unwrap();

    if let Err(e) = server.run("0.0.0.0:9000") {
        println!("run server err->{:?}", e);
    }
}
