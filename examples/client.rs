

use  bitcask::KvsClient;

fn main(){

    let mut client = KvsClient::connect("0.0.0.0:9000").unwrap();

    println!("start to set");
    for i in 0..100{
        let k = i.to_string();
        let v = i.to_string();
        
        match client.set(k, v){

            Ok(_)=>{
            },
            Err(e)=>{
                println!("set  err ->{:?}",e);
            }
        };
    }

    println!("start to get");
    for i in 0..100{
        let k = i.to_string();
        
        match client.get(k){

            Ok(r)=>{
                println!("get -> {:?}",r);
            },
            Err(e)=>{
                println!("get  err ->{:?}",e);
            }
        };
    }

    println!("start to remove");
    for i in 0..100{
        let k = i.to_string();
        
        match client.remove(k){

            Ok(r)=>{
                
            },
            Err(e)=>{
                println!("remove  err ->{:?}",e);
            }
        };
    }






}