use std::io::Error;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_big_array::BigArray;

use crate::{signature};
use crate::storage::keyvalue;

use super::Neighbors;


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
    pub fn execute(&self, src: String) -> Result<(), Error> {
        //add bussiness logic5
        Neighbors::add(src, self.public_key);
        Ok(())
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
enum Data {
    MessageType1(DataMessageType1),
}

impl Data {
    pub fn execute(&self, src: String) -> Result<(), Error> {
        match self {
            Data::MessageType1(data) => data.execute(src),
        }
    }

    pub fn generate(msg_type: u32) -> Self {
        match msg_type {
            1 => Data::MessageType1(DataMessageType1::generate()),
            _ => panic!("Invalid message type"),
        }
    }
}

impl Message {
    pub fn generate(message_type: u32) -> String {
        let timestamp = chrono::Utc::now().timestamp();
        let data: Data = match message_type {
            1 => Data::generate(message_type),
            _ => panic!("Invalid message type")
        };

        let signature = signature::new_signature(
            (timestamp.to_string()+&serde_json::to_string(&data).unwrap()).as_bytes(), 
            keyvalue::get(b"secret_key").unwrap().unwrap().as_slice());  
        serde_json::to_string(&Message {
            timestamp,
            data,
            signature
        }).unwrap()
    }

    fn verify(&self, src: String) -> Result<(), std::io::Error> {

        let has_pk = Neighbors::get(&src);
        match has_pk {
            Some(has_pk) => {
                let pk = has_pk.as_str().as_bytes();
                match signature::verify_signature((
                    self.timestamp.to_string()+&serde_json::to_string(&self.data).unwrap())
                        .as_bytes(), 
                    pk, &self.signature) {
                        Ok(()) => Ok(()),
                        Err(_) => {
                            let e = std::io::Error::new(std::io::ErrorKind::Other, "Invalid signature");
                            Err(e)
                        }
                    }
                },
            None => {
                let e = std::io::Error::new(std::io::ErrorKind::Other, "No public key found");
                Err(e)
            }
        }
    }
}