use std::time::Duration;

use color_eyre::eyre::Result;

mod transport;
mod signature;
mod storage;
mod communication;
mod validation;

use communication::messages::Packet;
use lazy_static::lazy_static;
use storage::keyvalue;
use storage::merkle;
use tokio::time::sleep;
use validation::block::Block;
use crate::communication::responses::Response;
use communication::{transactions::Transaction, messages::Message};

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

    pub static ref INIT_BLOCKCHAIN: bool = {
        let args: Vec<String> = std::env::args().collect();
        match args.iter().position(|arg| arg == "--init") {
            Some(_) => true,
            None => false,
        }
    };

    pub static ref GENERATE_NEW_PAIR: bool = {
        let args: Vec<String> = std::env::args().collect();
        match args.iter().position(|arg| arg == "--gen-pair") {
            Some(_) => true,
            None => false,
        }
    };
}

#[tokio::main]
async fn main() -> Result<()> {

    color_eyre::install()?;

    if GENERATE_NEW_PAIR.to_owned(){
        storage::keyvalue::insert(b"secret_key", &signature::new_pair().0).unwrap();
    }

    println!("Listening on port {}", PORT_NUMBER.to_owned());
    tokio::spawn(
        transport::listen(
            Box::new(|bytes: &Vec<u8>, src: String| {
            let request: Packet = serde_json::from_slice(bytes).unwrap();
            println!("Received from {:?} --> {:?}\n", src, serde_json::to_string(&request).unwrap());

            let response: Option<Response> = match request {
                Packet::Message(msg) => Some(match msg.execute(src.clone()) {
                    Ok(response) => {
                        println!("Sending to {:?} --> {:?}\n", src.clone(), serde_json::to_string(&response).unwrap());
                        response
                    },
                    Err(err) => {
                        println!("Error: {:?}", err);
                        Response::generate(500).unwrap()
                    }
                }),
                Packet::Response(resp) => {
                    resp.execute(src.clone()).unwrap();
                    None
                },
            };

            match response {
                Some(resp) => {
                    let response_bytes = serde_json::to_string(&resp).unwrap();
                    Some(response_bytes)
                }
                None => None,
            }
            })
        )
    );

    if INIT_BLOCKCHAIN.to_owned() == true {

        println!("Initializing blockchain...");
        create_new_blockchain().await;
    }
    if PEER_ADDR.to_owned().len() > 0 {

        sleep(Duration::from_secs(5)).await;

        println!("Sending");

        let transaction = Transaction::generate_with_vec(2, b"Hello World".to_vec());

        transport::send(PEER_ADDR.to_string(), 
        serde_json::to_string(&Packet::Message(Message::generate_with_transaction(3, transaction))).unwrap()).await.unwrap();

    }
    loop {}

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

async fn create_new_blockchain() {
    storage::keyvalue::insert(b"secret_key", &signature::new_pair().0).unwrap();
    merkle::reset_local_trie().await;
    let first_transaction = Transaction::generate(6);
    // let transaction: Transaction = serde_json::from_str(&first_transaction).unwrap();
    merkle::insert(&first_transaction.signature, 
        serde_json::to_vec(&first_transaction).unwrap().as_slice()).await;
    let initial_block: Block = Block::create_from_loca_trie().await;
    keyvalue::insert(&initial_block.header, serde_json::to_vec(&initial_block).unwrap().as_slice()).unwrap();
    keyvalue::insert(b"last_block_header", &initial_block.header).unwrap();
}