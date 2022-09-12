

use bitcask::KvServer;
use bitcask::Engine;
use std::path::PathBuf;
use bitcask::NaiveThreadPool;

fn main(){

    let path:PathBuf = "./testdata".into();

    let server:KvServer<NaiveThreadPool> = KvServer::new(Engine::SLED,Some(path)).unwrap();

    if let Err(e) = server.run("0.0.0.0:9000"){

        println!("run server err->{:?}",e);
    }

}