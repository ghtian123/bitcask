use bitcask::AsyncKvsClient;

#[tokio::main]
async fn main() {
    let mut client = AsyncKvsClient::connect("0.0.0.0:9000").await.unwrap();

    println!("start to set");
    for i in 0..100 {
        let k = i.to_string();
        let v = i.to_string();

        client.set(k, v).await.unwrap();
    }

    println!("start to get");
    for i in 0..100 {
        let k = i.to_string();

        client.get(k).await.unwrap();
    }

    println!("start to remove");
    for i in 0..100 {
        let k = i.to_string();

        client.remove(k).await.unwrap();
    }

    // sleep(std::time::Duration::from_secs(10));
}
