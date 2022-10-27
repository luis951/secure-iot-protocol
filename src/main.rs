use std::time::Duration;

use color_eyre::eyre::Result;

mod transport;
mod signature;
mod storage;
mod communication;
mod validation;
mod testing;

use communication::messages::Packet;
use lazy_static::lazy_static;
use storage::keyvalue;
use storage::merkle;
use tokio::time::sleep;
use validation::block::Block;
use crate::communication::responses::Response;
use communication::{transactions::Transaction, transactions::TransactionData, messages::Message, neighbors::Neighbors};
use validation::block::{LocalBlock};

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

    pub static ref EXECUTE_TESTS: bool = {
        let args: Vec<String> = std::env::args().collect();
        match args.iter().position(|arg| arg == "--test") {
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
        transport::listen()
    );

    if EXECUTE_TESTS.to_owned() == true {
        testing::transport_tests().await;
        testing::signature_tests();
        testing::storage_tests();
        return Ok(());
    }

    if INIT_BLOCKCHAIN.to_owned() == true {

        println!("Initializing blockchain...");
        create_new_blockchain().await;
    }
    if PEER_ADDR.to_owned().len() > 0 {

        // println!("Sending");

        // let handshake = Packet::Message(Message::generate(1));
        // transport::send(PEER_ADDR.to_string(), serde_json::to_string(&handshake).unwrap()).await;

        // match transport::send(PEER_ADDR.clone(), 
        //     serde_json::to_string(&Packet::Message(
        //         Message::generate(5)
        //     )).unwrap()
        // ).await {
        //     Ok(_) => {
        //         println!("message type 5 sent");},
        //     Err(e) => {
        //         println!("error sending message: {}", e);
        //     },
        // }

        // for t in 0..10  {
        //     println!("Sending transaction {}", t);
        //     let transaction = Transaction::generate_with_vec(2, ("Hello World ".to_owned()+&t.to_string()).as_bytes().to_vec());
        //     LocalBlock::insert_transaction(transaction.clone()).await;
        //     let message = Packet::Message(Message::generate_with_transaction(3, transaction.clone()));
            
        //     for (addr, _) in Neighbors::restore().neighbors {
        //         println!("Sending to {}", addr);
        //         transport::send(addr, serde_json::to_string(&message).unwrap()).await;
        //     }
        // }
    }
    loop {
        println!("Menu:");
        println!("1. Enviar mensagem de conexão");
        println!("2. Enviar transação");
        println!("3. Solicitar estado atual da blockchain");
        println!("4. Buscar transação pelo cabeçalho");

        let mut unformatted_input = String::new();
        std::io::stdin().read_line(&mut unformatted_input).unwrap();
        let input: u8;
        match unformatted_input.trim().parse() {
            Ok(num) => input = num,
            Err(_) => {
                println!("Opção inválida");
                continue
            },
        }
        match input {
            1 => {
                let message = Packet::Message(Message::generate(1));
                transport::send(PEER_ADDR.to_string(), serde_json::to_string(&message).unwrap()).await;
            },
            2 => {
                println!("Insira dados de transação:");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let transaction = Transaction::generate_with_vec(2, (input).as_bytes().to_vec());
                LocalBlock::insert_transaction(transaction.clone()).await;
                let message = Packet::Message(Message::generate_with_transaction(3, transaction.clone()));
                for (addr, _) in Neighbors::restore().neighbors {
                    println!("Enviando para {}", addr);
                    transport::send(addr, serde_json::to_string(&message).unwrap()).await;
                }
            },
            3 => {
                let message = Packet::Message(Message::generate(5));
                transport::send(PEER_ADDR.to_string(), serde_json::to_string(&message).unwrap()).await;
            },
            4 => {
                println!("Insira cabeçalho de transação:");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                input = input.trim().to_string();
                let header = hex::decode(input).unwrap();
                let transaction = Block::search_transaction_in_blockchain(header.as_slice());
                match transaction {
                    Some(t) => {
                        println!("Transação:");
                        println!("Timestamp: {:?}", t.timestamp);
                        match t.data {
                            TransactionData::Type2(t_data) => {
                                println!("Tipo da Transação: 2");
                                println!("Data: {:?}", String::from_utf8(t_data.data));
                            },
                            TransactionData::Type6(_) => {
                                println!("Transação Tipo: 6");
                                println!("Contém chave pública de assinatura federada");
                            },
                            _ => {
                                println!("Transação Tipo: outro");
                            },
                        }
                    },
                    None => {
                        println!("Transação não encontrada");
                    },
                }
            },
            _ => {
                println!("Cabeçalho inválido");
            },
        }
    }

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
    merkle::reset_local_trie().await;
    let first_transaction = Transaction::generate(6);
    // let transaction: Transaction = serde_json::from_str(&first_transaction).unwrap();
    merkle::insert(&first_transaction.signature, 
        serde_json::to_vec(&first_transaction).unwrap().as_slice()).await;
    let initial_block: Block = Block::create_from_local_trie().await;
    initial_block.save_to_blockchain();
    
}