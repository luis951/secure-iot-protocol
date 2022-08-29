use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

use crate::{storage::{merkle, self}, signature};

#[derive(Serialize, Deserialize)]
enum FederationSignature {
    #[serde(with = "BigArray")]
    Signed([u8; 64]),
    Unsigned,
}

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub header: Vec<u8>,
    #[serde(with = "BigArray")]
    issuer: [u8; 33],
    body: Vec<u8>,
    timestamp: i64,
    previous_block_header: Vec<u8>,
    #[serde(with = "BigArray")]
    issuer_signature: [u8; 64],
    federation_signature: FederationSignature,
}

impl Block {
    pub async fn create_from_loca_trie() -> Block {
        let timestamp = chrono::Utc::now().timestamp();
        let (body, header) = merkle::get_local_trie().await;
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
    
}