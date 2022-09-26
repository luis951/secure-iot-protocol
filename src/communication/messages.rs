use std::io::{Error, ErrorKind};

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::signature;
use crate::storage::{keyvalue, merkle};
use crate::validation::block::{self, Block};

use super::responses::Response;
use super::neighbors::{Neighbors, Node};
use super::transactions::Transaction;

#[derive(Serialize, Deserialize)]
pub enum Packet {
    Message(Message),
    Response(Response),
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    timestamp: i64,
    data: Data,
    #[serde(with = "BigArray")]
    signature: [u8; 64]
}

#[derive(Serialize, Deserialize)]
struct DataMessageType1 {
    #[serde(with = "BigArray")]
    public_key: [u8; 33]
}

impl DataMessageType1 {
    pub fn execute(&self, src: String) -> Result<Response, Error> {
        //TODO: add bussiness logic (block too many node connections, verify node type)
        Neighbors::add(src, Node{pk: self.public_key, is_validator: false});

        Ok(Response::generate(2).unwrap())
    }

    pub fn generate() -> Self {
        let sk = keyvalue::get(b"secret_key").unwrap().unwrap();
        let public_key = signature::generate_public_key(sk.as_slice());
        let data = DataMessageType1 {
            public_key
        };
        data
    }
}

#[derive(Serialize, Deserialize)]
struct DataMessageType2 {
}

impl DataMessageType2 {
    pub fn execute(&self, _src: String) -> Result<Response, Error> {
        //TODO: add bussiness logic (block too many node connections, verify node type)
        // let nodes = self.neighbors.neighbors;
        // for (addr, _) in nodes {
        //     transport::send(node, addr, msg);
        // }
        Ok(Response::generate(1).unwrap())
    }

    // pub fn generate() -> Self {
    //     let sk = keyvalue::get(b"secret_key").unwrap().unwrap();
    //     let public_key = signature::generate_public_key(sk.as_slice());
    //     let data = DataMessageType1 {
    //         public_key
    //     };
    //     data
    // }
}

#[derive(Serialize, Deserialize)]
struct DataMessageType3 {
    transaction: Transaction
}

impl DataMessageType3 {
    pub async fn execute(&self) -> Result<Response, Error> {
        // TODO: verify more transaction details (address balance)
        match block::LocalBlock::insert_transaction(self.transaction.clone()).await {
            Ok(()) => Ok(Response::generate(1).unwrap()),
            Err(_) => Ok(Response::generate(500).unwrap())
        }
    }

    pub fn generate(transaction: Transaction) -> Self {
        let data = DataMessageType3 {
            transaction
        };
        data
    }

}

#[derive(Serialize, Deserialize)]
struct DataMessageType4 {
    block: Block
}

impl DataMessageType4 {
    pub async fn execute(&self) -> Result<Response, Error> {
        // TODO: verify block details (transactions missing or added)

        self.block.save_to_blockchain();

        // keyvalue::insert(&self.block.header, serde_json::to_vec(&self.block).unwrap().as_slice()).unwrap();
        // keyvalue::insert(b"last_block_header", &self.block.header).unwrap();
        Ok(Response::generate(1).unwrap())
    }

    pub fn generate(block: Block) -> Self {
        let data = DataMessageType4 {
            block
        };
        data
    }

}

#[derive(Serialize, Deserialize)]
struct DataMessageType5 {
    until_header: Vec<u8>
}

impl DataMessageType5 {

    pub fn generate() -> Self {
        let until_header = match keyvalue::get(b"last_block_header").unwrap() {
            Some(header) => header,
            None => vec![]
        };
        let data = DataMessageType5 {
            until_header
        };
        data
    }

    pub fn execute(&self) -> Result<Response, Error> {
        let mut blocks: Vec<Block> = Vec::new();

        let mut header = keyvalue::get(b"last_block_header").unwrap().unwrap();
        let mut block_serialized = keyvalue::get(&header).unwrap().unwrap();
        let mut block: Block = serde_json::from_slice(&block_serialized.as_slice()).unwrap();
        blocks.push(block.clone());
        while block.previous_block_header.len() > 0 {
            header = block.previous_block_header.clone();
            block_serialized = keyvalue::get(&header).unwrap().unwrap();
            block = serde_json::from_slice(&block_serialized.as_slice()).unwrap();
            
            if block.header == self.until_header {
                break;
            }

            blocks.push(block.clone());
        }
        let response = Response::generate_with_block_vector(3, blocks).unwrap();
        Ok(response)
    }
}


#[derive(Serialize, Deserialize)]
enum Data {
    MessageType1(DataMessageType1), // Send public key for safe communication
    MessageType2(DataMessageType2), // Request peer neighboring nodes list
    MessageType3(DataMessageType3), // Propagate transaction to neighbor
    MessageType4(DataMessageType4), // Propagate new block to neighbor
    MessageType5(DataMessageType5), // Request current blockchain state
}

impl Data {
    async fn execute(&self, src: String) -> Result<Response, Error> {
        match self {
            Data::MessageType1(data) => data.execute(src),
            Data::MessageType2(data) => data.execute(src),
            Data::MessageType3(data) => data.execute().await,
            Data::MessageType4(data) => data.execute().await,
            Data::MessageType5(data) => data.execute(),
            _ => Err(Error::new(ErrorKind::Unsupported, "Unsupported message type"))
        }
    }

    fn generate(msg_type: u32) -> Self {
        match msg_type {
            1 => Data::MessageType1(DataMessageType1::generate()),
            // 2 => Data::MessageType2(DataMessageType2::generate()),
            5 => Data::MessageType5(DataMessageType5::generate()),
            _ => panic!("Invalid message type"),
        }
    }

    fn generate_with_transaction(msg_type: u32, transaction: Transaction) -> Self {
        match msg_type {
            3 => Data::MessageType3(DataMessageType3::generate(transaction)),
            _ => panic!("Invalid message type"),
        }
    }

    fn generate_with_block(msg_type: u32, block: Block) -> Self {
        match msg_type {
            4 => Data::MessageType4(DataMessageType4::generate(block)),
            _ => panic!("Invalid message type"),
        }
    }
}

impl Message {
    pub fn generate(message_type: u32) -> Self {
        let timestamp = chrono::Utc::now().timestamp();

        let data: Data = match message_type {
            1 => Data::generate(message_type),
            5 => Data::generate(message_type),
            _ => panic!("Invalid message type")
        };

        let signature = signature::new_signature(
            (timestamp.to_string()+&serde_json::to_string(&data).unwrap()).as_bytes(), 
            keyvalue::get(b"secret_key").unwrap().unwrap().as_slice());
        Message {
            timestamp,
            data,
            signature
        }
    }

    pub fn generate_with_transaction(message_type: u32, transaction: Transaction) -> Self {
        let timestamp = chrono::Utc::now().timestamp();

        let data: Data = match message_type {
            3 => Data::generate_with_transaction(message_type, transaction),
            _ => panic!("Invalid message type")
        };

        let signature = signature::new_signature(
            (timestamp.to_string()+&serde_json::to_string(&data).unwrap()).as_bytes(), 
            keyvalue::get(b"secret_key").unwrap().unwrap().as_slice());  
        Message {
            timestamp,
            data,
            signature
        }
    }

    pub async fn generate_with_block(message_type: u32, block: Block) -> Self {
        let timestamp = chrono::Utc::now().timestamp();

        let data: Data = match message_type {
            4 => Data::generate_with_block(message_type, block),
            _ => panic!("Invalid message type")
        };

        let signature = signature::new_signature(
            (timestamp.to_string()+&serde_json::to_string(&data).unwrap()).as_bytes(), 
            keyvalue::get(b"secret_key").unwrap().unwrap().as_slice());  
        Message {
            timestamp,
            data,
            signature
        }
    }

    pub async fn execute(&self, src: String) -> Result<Response, Error> {
        match self.verify(src.clone()) {
            Ok(_) => {
                println!("Verified message from {}", src);
                let data = self.data.execute(src.clone()).await;
                data
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    match &self.data {
                        Data::MessageType1(data) => {
                            return data.execute(src)
                        },
                        _ => Err(e)
                    }
                }
                _ => {
                    println!("{}", e);
                    Err(e)
                }
            }
        }
    }

    fn verify(&self, src: String) -> Result<(), std::io::Error> {
        // TODO: verify timestamp
        let has_pk = Neighbors::get(&src);
        match has_pk {
            Some(node) => {
                let pk = node.pk;
                match signature::verify_signature((
                    self.timestamp.to_string()+&serde_json::to_string(&self.data).unwrap())
                        .as_bytes(), 
                    &pk, &self.signature) {
                        Ok(()) => Ok(()),
                        Err(_) => {
                            let e = std::io::Error::new(std::io::ErrorKind::Other, "Invalid signature");
                            Err(e)
                        }
                    }
                },
            None => {
                let e = std::io::Error::new(std::io::ErrorKind::NotFound, "No public key found");
                Err(e)
            }
        }
    }
}