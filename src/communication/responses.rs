use std::io::Error;

use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

use crate::{storage::keyvalue, signature, validation::block::Block};

use super::neighbors::{Node, Neighbors};


#[derive(Serialize, Deserialize)]
struct Type2Data {
    #[serde(with = "BigArray")]
    pk: [u8; 33],
}

impl Type2Data {
    fn execute(&self, src: String) -> Result<(), Error> {
        println!("add node: {}", src);
        let new_node = Node {
            pk: self.pk,
            is_validator: true, // TODO: add check after blockchain is received
        };
        Neighbors::add(src, new_node);
        Ok(())
    }

    fn generate() -> Self {
        let sk = keyvalue::get(b"secret_key").unwrap().unwrap();
        let pk = signature::generate_public_key(sk.as_slice());
        let data = Type2Data {
            pk,
        };
        data
    }
}

#[derive(Serialize, Deserialize)]
struct Type3Data {
    blocks: Vec<Block>,
}

impl Type3Data {
    fn execute(&self, _src: String) -> Result<(), Error> {
        self.blocks.clone().into_iter().rev().for_each(|block| {
            block.save_to_blockchain()
        });
        Ok(())
    }

    fn generate(blocks: Vec<Block>) -> Self {
        let data = Type3Data {
            blocks,
        };
        data
    }
}

#[derive(Serialize, Deserialize)]
enum Data {
    ResponseType1, // Ok response, do nothing
    ResponseType2(Type2Data), // Added to node list, data is public key
    ResponseType3(Type3Data), // Send current blockchain state
    ErrorResponse // Error response, do nothing
}

impl Data {
    fn execute(&self, src: String) -> Result<(), Error> {
        print!("execute response");
        match self {
            Data::ResponseType1 => Ok(()),
            Data::ResponseType2(data) => data.execute(src),
            Data::ResponseType3(data) => data.execute(src),
            Data::ErrorResponse => Ok(()),
        }
    }

    fn generate(data_type: u32) -> Result<Self, Error> {
        match data_type {
            1 => Ok(Data::ResponseType1),
            2 => Ok(Data::ResponseType2(Type2Data::generate())),
            500 => Ok(Data::ErrorResponse),
            _ => panic!("Invalid message type"),
        }
    }

    fn generate_with_block_vector(data_type: u32, blocks: Vec<Block>) -> Result<Self, Error> {
        match data_type {
            3 => Ok(Data::ResponseType3(Type3Data::generate(blocks))),
            _ => panic!("Invalid message type"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    timestamp: i64,
    data: Data,
    #[serde(with = "BigArray")]
    signature: [u8; 64]
}

impl Response {
    pub fn execute(&self, src: String) -> Result<(), Error> {
        self.data.execute(src)
    }

    pub fn generate(data_type: u32) -> Result<Self, Error> {
        match Data::generate(data_type) {
            Ok(data) => {
                let timestamp = chrono::Utc::now().timestamp();

                let signature = signature::new_signature(
                    (timestamp.to_string()+&serde_json::to_string(&data).unwrap()).as_bytes(), 
                    keyvalue::get(b"secret_key").unwrap().unwrap().as_slice());  

                Ok(Response {
                    timestamp,
                    data,
                    signature,
                })
            }, 
            Err(err) => Err(err),
        }
    }

    pub fn generate_with_block_vector(data_type: u32, blocks: Vec<Block>) -> Result<Self, Error> {
        match Data::generate(data_type) {
            Ok(data) => {
                let timestamp = chrono::Utc::now().timestamp();

                let signature = signature::new_signature(
                    (timestamp.to_string()+&serde_json::to_string(&data).unwrap()).as_bytes(), 
                    keyvalue::get(b"secret_key").unwrap().unwrap().as_slice());  

                Ok(Response {
                    timestamp,
                    data,
                    signature,
                })
            }, 
            Err(err) => Err(err),
        }
    }
}
