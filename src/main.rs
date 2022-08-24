use std::time::Duration;

use color_eyre::eyre::Result;

mod transport;
mod signature;
mod storage;
mod communication;

use lazy_static::lazy_static;
use storage::keyvalue;
use storage::merkle;
use tokio::time::sleep;

const DB_PATH: &str = "./storage.db";

lazy_static!{
    pub static ref PORT_NUMBER: String = {
        let args: Vec<String> = std::env::args().collect();
        match args.iter().position(|arg| arg == "-p" || arg == "--port") {
            Some(i) => {
                match args.get(i + 1) {
                    Some(port) => port.to_string(),
                    None => panic!("No port number provided"),
                }
            }
            None => "8640".to_string(),
        }
    };

    pub static ref PEER_ADDR: String = {
        let args: Vec<String> = std::env::args().collect();
        match args.iter().position(|arg| arg == "-x" || arg == "--peer") {
            Some(i) => {
                match args.get(i + 1) {
                    Some(port) => port.to_string(),
                    None => panic!("No port number provided"),
                }
            }
            None => {
                println!("No address provided");
                "".to_string()},
        }
    };
}

#[tokio::main]
async fn main() -> Result<()> {

    // println!("{}",communication::messages::Message::generate(1));

    color_eyre::install()?;
    tokio::spawn(
        transport::listen(
            Box::new(|bytes: &Vec<u8>, src: String| {
            let message:communication::messages::Message = serde_json::from_slice(bytes).unwrap();
            // println!("received {:?}", serde_json::to_string(&message));
            match message.execute(src) {
                Ok(response) => {
                    Some("200".to_string())
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    Some("400".to_string())
                }
            }
            })
        )
    );

    sleep(Duration::from_secs(5)).await;

    println!("Sending");

    storage::keyvalue::insert(b"secret_key", &signature::new_pair().0).unwrap();

    transport::send(PEER_ADDR.to_string(), 
    communication::messages::Message::generate(1).to_string()).await.unwrap();

    // loop {}

    // let (sk, pk) = signature::new_pair();
    // let msg = b"hello world";
    // let signature = signature::new_signature(msg, &sk);
    // print!("{:?}",verify_signature(msg, &pk, &signature));

    // let trie = merkle::create_new();

    // //STORAGE ROCKSDB TEST

    // let database = keyvalue::open_db("./general-data");

    // database.put(b"key", b"value").unwrap();
    // database.get(b"key").unwrap();

    //TRANSPORT TEST
    

    
    // let (node3, inc_3) = transport::create_node(4435).await?;
    // let (node4, inc_4) = transport::create_node(4436).await?;
    // tokio::spawn(transport::callback(node2.clone(), inc_2));
    // tokio::spawn(transport::callback(node3.clone(), inc_3));
    // tokio::spawn(transport::callback(node4.clone(), inc_4));

    
    // transport::client(&node3, node2.local_addr().to_string().as_str(), "teste").await?;
    // transport::client(&node2, node3.local_addr().to_string().as_str(), "teste").await?;
    // transport::client(&node1, node4.local_addr().to_string().as_str(), "teste").await?;

    
    // node2.close();
    // node3.close();
    // node4.close();

    Ok(())
}