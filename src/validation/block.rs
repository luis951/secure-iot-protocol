use std::io::{Error, ErrorKind};

use async_recursion::async_recursion;
use lazy_static::__Deref;
use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

use crate::{storage::{merkle, self, keyvalue}, signature, communication::{transactions::Transaction, neighbors::{self, Node, Neighbors}, messages::{Packet, Message}}, transport};

#[derive(Serialize, Deserialize, Clone)]
enum FederationSignature {
    #[serde(with = "BigArray")]
    Signed([u8; 64]),
    Unsigned,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub header: Vec<u8>,
    #[serde(with = "BigArray")]
    issuer: [u8; 33],
    body: Vec<u8>,
    timestamp: i64,
    pub previous_block_header: Vec<u8>,
    #[serde(with = "BigArray")]
    issuer_signature: [u8; 64],
    federation_signature: FederationSignature,
}

impl Block {
    pub async fn create_from_local_trie() -> Block {
        let timestamp = chrono::Utc::now().timestamp();
        let (header, body) = merkle::get_local_trie().await;
        let sk = storage::keyvalue::get(b"secret_key").unwrap().unwrap();
        let issuer = signature::generate_public_key(&sk);
        let previous_block_header = match storage::keyvalue::get(b"last_block_header").unwrap() {
            Some(header) => header,
            None => vec![],
        };
        

        let issuer_signature = signature::new_signature(
            (timestamp.to_string()+&serde_json::to_string(&header).unwrap()).as_bytes(), 
            sk.as_slice());

        let federation_signature = match storage::keyvalue::get(b"federated_secret_key").unwrap() {
            Some(federated_sk) => FederationSignature::Signed(
                signature::new_signature(
                    (timestamp.to_string()+&serde_json::to_string(&header).unwrap()).as_bytes(), 
                    federated_sk.as_slice())
            ),
            None => FederationSignature::Unsigned,
        };

        Block {
            header,
            issuer,
            body,
            timestamp,
            previous_block_header,
            issuer_signature,
            federation_signature,
        }
    }

    pub fn save_to_blockchain(&self){
        keyvalue::insert(&self.header, serde_json::to_vec(&self).unwrap().as_slice()).unwrap();
        keyvalue::insert(b"last_block_header", &self.header).unwrap();
        Block::print_blockchain();
    }

    pub fn print_block(&self) {
        let trie = merkle::create_evaluation_trie(self.body.clone(), self.header.clone());
        println!("HEADER: {:?}", self.header.clone());
        println!("ISSUER: {:?}", self.issuer);
        println!("TIMESTAMP: {:?}", self.timestamp);
        println!("PREVIOUS BLOCK HEADER: {:?}", self.previous_block_header);
        println!("BODY: ");
        for (key, value) in trie.iter() {
            let t: Transaction = serde_json::from_slice(&value.as_slice()).unwrap();
            println!("{}: {}", hex::encode(key), serde_json::to_string(&t).unwrap());
        }
        println!("-----------------------------------\n\n")
    }

    pub fn print_blockchain() {
        println!("\n\nPRINT FULL BLOCKCHAIN");
        let mut header = keyvalue::get(b"last_block_header").unwrap().unwrap();
        let mut block_serialized = keyvalue::get(&header).unwrap().unwrap();
        let mut block: Block = serde_json::from_slice(&block_serialized.as_slice()).unwrap();
        block.print_block();
        while block.previous_block_header.len() > 0 {
            header = block.previous_block_header.clone();
            block_serialized = keyvalue::get(&header).unwrap().unwrap();
            block = serde_json::from_slice(&block_serialized.as_slice()).unwrap();
            block.print_block();
        }
    }

    #[async_recursion]
    pub async fn send_block(self, peer: Option<String>) {
        let packet = Packet::Message(Message::generate_with_block(4, self).await);
        let serialized_packet = serde_json::to_string(&packet).unwrap();
        match peer {
            Some(addr) => transport::send(addr, serialized_packet.clone()).await.unwrap(),
            None => {
                let mut neighbors = Neighbors::restore();
                for (addr, _) in neighbors.neighbors.iter() {
                    transport::send(addr.to_string(), serialized_packet.clone()).await.unwrap();
                }
            }
        }
    }
    
}

pub struct LocalBlock {
}

impl LocalBlock {
    pub async fn insert_transaction(transaction: Transaction) -> Result<(), Error> {
        // TODO: verify transaction data in block
        match signature::verify_signature((transaction.timestamp.to_string()+
                                &serde_json::to_string(&transaction.data).unwrap()).as_bytes(), 
            &transaction.pk, &transaction.signature) {
                Ok(()) => {
                    println!("OK");
                    merkle::insert(&transaction.signature, &serde_json::to_vec(&transaction).unwrap()).await;
                    let mut t_n = merkle::LOCAL_BLOCK_SIZE.write().await;
                    *t_n += 1;
                    if *t_n == 10 {
                        println!("SENDING BLOCK AND RESETTING");
                        let block = Block::create_from_local_trie().await;
                        block.clone().save_to_blockchain();
                        block.clone().send_block(None).await;
                    }
                    Ok(())
                },
                Err(e) => {
                    println!("{}", e);
                    Err(Error::new(ErrorKind::InvalidData, e))},
            }
    }
}