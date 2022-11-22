use std::io::{Error, ErrorKind};

use async_recursion::async_recursion;
use lazy_static::__Deref;
use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;
use serde_json::Value;
use super::address::{Address, AddressesState};

use crate::{storage::{merkle, self, keyvalue}, signature, communication::{transactions::{Transaction, DataTransactionType7, TransactionData}, neighbors::{self, Node, Neighbors}, messages::{Packet, Message}}, transport, INIT_BLOCKCHAIN};

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
    pub addresses_state: AddressesState,
    pub body: Vec<u8>,
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
        
        let addresses_state = {
        
            let mut previous_state = match previous_block_header.len(){
                0 => {AddressesState::new()},
                _ => {
                    let previous_block: Block = serde_json::from_slice(storage::keyvalue::get(previous_block_header.clone().as_slice()).unwrap().unwrap().as_slice()).unwrap();
                    let previous_state = previous_block.addresses_state;
                    previous_state
                }
            };

            let all_transactions = merkle::get_all().await;
            for t in all_transactions {
                let transaction: Transaction = serde_json::from_slice(t.1.as_slice()).unwrap();
                let last_transaction = transaction.signature;
                let mut balance: i64 = 0;
                let mut linked_addresses = vec![];
                if previous_state.state.contains_key(&hex::encode(transaction.pk)) {

                    match transaction.data {
                        TransactionData::Type7(data) => {
                            let sent_address_pk = data.recipient_pk;
                            let mut sent_address_current_balance = 0;
                            let mut sent_address_linked_addresses = vec![];
                            let mut sent_address_last_transaction: [u8; 64] = [0; 64];
                            if previous_state.state.contains_key(&hex::encode(sent_address_pk)) {
                                let found_address:Address = serde_json::from_str(previous_state.state.get(&hex::encode(sent_address_pk)).unwrap().as_str().unwrap()).unwrap();
                                sent_address_last_transaction = found_address.last_transaction;
                                sent_address_linked_addresses = found_address.linked_addresses;
                                sent_address_current_balance = found_address.balance;
                                
                            }
                            let sent_address = Address{
                                last_transaction: sent_address_last_transaction,
                                balance: sent_address_current_balance + data.balance_variation,
                                linked_addresses: sent_address_linked_addresses,
                            };
                            previous_state.state.insert(hex::encode(sent_address_pk), Value::String(serde_json::to_string(&sent_address).unwrap()));
                        },
                        _ => {}
                    }

                    let value = previous_state.state.get(&hex::encode(transaction.pk)).unwrap()
                    .to_owned().as_str().unwrap().replace("//", "");
                    let found_address:Address = serde_json::from_str(&value).unwrap();
                    balance = found_address.balance;
                    linked_addresses = found_address.linked_addresses;
                }

                balance += transaction.balance_variation;

                let new_address_data:Address = Address{
                    balance,
                    linked_addresses,
                    last_transaction,
                };
                
                previous_state.state.insert(hex::encode(transaction.pk), 
                    Value::String(serde_json::to_string(&new_address_data).unwrap()));
            }

            previous_state
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
            addresses_state,
            body,
            timestamp,
            previous_block_header,
            issuer_signature,
            federation_signature,
        }
    }

    pub fn save_to_blockchain(&self){
        let value = serde_json::to_vec(&self).unwrap();
        keyvalue::insert(self.header.as_slice(), value.as_slice()).unwrap();
        keyvalue::insert(b"last_block_header", &self.header).unwrap();
        Block::print_blockchain();
    }

    pub fn print_block(&self) {
        let trie = merkle::create_evaluation_trie(self.body.clone(), self.header.clone());
        println!("HEADER: {:?}", self.header.clone());
        println!("ISSUER: {:?}", self.issuer);
        println!("TIMESTAMP: {:?}", self.timestamp);
        println!("PREVIOUS BLOCK HEADER: {:?}", self.previous_block_header);
        println!("ADDRESSES STATE:");
        for (key, value) in self.addresses_state.state.iter() {
            println!("{}: {}", key, value);
        }

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

    pub fn search_transaction_in_blockchain(signature: &[u8]) -> Option<Transaction> {

        let mut header = keyvalue::get(b"last_block_header").unwrap().unwrap();
        let mut block_serialized = keyvalue::get(&header).unwrap().unwrap();
        let mut block: Block = serde_json::from_slice(&block_serialized.as_slice()).unwrap();
        let trie = merkle::create_evaluation_trie(block.body, block.header);
        
        while block.previous_block_header.len() > 0 && !trie.contains(signature).unwrap() {
            header = block.previous_block_header.clone();
            let mut block_serialized = keyvalue::get(&header).unwrap().unwrap();
            let mut block: Block = serde_json::from_slice(&block_serialized.as_slice()).unwrap();
            let trie = merkle::create_evaluation_trie(block.body, block.header);
        }
        if trie.contains(&signature).unwrap() {
            Some(serde_json::from_slice(trie.get(signature).unwrap().unwrap().as_slice()).unwrap())
        } else {
            None
        }
    }

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
                        if INIT_BLOCKCHAIN.to_owned() {
                            let block = Block::create_from_local_trie().await;
                            block.clone().save_to_blockchain();
                            block.clone().send_block(None).await;

                            *t_n = 0;
                            merkle::reset_local_trie().await;
                        }
                    }
                    Ok(())
                },
                Err(e) => {
                    println!("{}", e);
                    Err(Error::new(ErrorKind::InvalidData, e))},
            }
    }
}