use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

use crate::{storage::keyvalue, signature};

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub timestamp: i64,
    #[serde(with = "BigArray")]
    pub pk: [u8; 33],
    pub data: TransactionData,
    pub balance_variation: i64,
    #[serde(with = "BigArray")]
    pub signature: [u8; 64],
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TransactionData {
    Type1(DataTransactionType1), // register child address
    Type2(DataTransactionType2), // record data
    Type6(DataTransactionType6), // generate new blockchain federated signing pair
    Type7(DataTransactionType7), // transfer value from balance
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DataTransactionType1 {
    #[serde(with = "BigArray")]
    child_pk: [u8; 33],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DataTransactionType2 {
    pub data: Vec<u8>,
}

impl DataTransactionType2 {
    pub fn generate(data: Vec<u8>) -> Self {
        DataTransactionType2 {
            data
        }
    }
}

// must be the first transaction in a block chain
// pk will be used to validate all subsequent block
#[derive(Serialize, Deserialize, Clone)]
pub struct DataTransactionType6 {
    #[serde(with = "BigArray")]
    federated_pk: [u8; 33],
}

impl DataTransactionType6 {
    pub fn generate() -> Self {
        let federated_pair = signature::new_pair();
        let federated_pk = federated_pair.1;
        keyvalue::insert(b"federated_secret_key", &federated_pair.0).unwrap();
        DataTransactionType6 {
            federated_pk
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DataTransactionType7 {
    #[serde(with = "BigArray")]
    pub recipient_pk: [u8; 33],
    pub balance_variation: i64,
}

impl Transaction {
    pub fn generate(transaction_type: u32) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let sk = keyvalue::get(b"secret_key").unwrap().unwrap();
        let pk = signature::generate_public_key(sk.as_slice());
        let (data, balance_variation): (TransactionData,i64) = match transaction_type {
            6 => {
                let data = DataTransactionType6::generate();
                (TransactionData::Type6(data), std::i64::MAX)
            }
            _ => {
                panic!("invalid transaction type");
            }
        };
        let signature = signature::new_signature(
            (timestamp.to_string()+&serde_json::to_string(&data).unwrap()).as_bytes(), 
            sk.as_slice());

        Transaction {
            timestamp,
            pk,
            data,
            balance_variation,
            signature,
        }
    }

    pub fn generate_with_vec(transaction_type: u32, data: Vec<u8>) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let sk = keyvalue::get(b"secret_key").unwrap().unwrap();
        let pk = signature::generate_public_key(sk.as_slice());
        let (data, balance_variation): (TransactionData,i64) = match transaction_type {
            2 => {
                let transaction_data = DataTransactionType2::generate(data);
                (TransactionData::Type2(transaction_data), 1)
            }
            _ => {
                panic!("invalid transaction type");
            }
        };
        let signature = signature::new_signature(
            (timestamp.to_string()+&serde_json::to_string(&data).unwrap()).as_bytes(), 
            sk.as_slice());

        Transaction {
            timestamp,
            pk,
            data,
            balance_variation,
            signature,
        }
    }

    fn generate_vec_and_i64(transaction_type: u32, data: Vec<u8>, value: i64) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let sk = keyvalue::get(b"secret_key").unwrap().unwrap();
        let pk = signature::generate_public_key(sk.as_slice());
        let (data, balance_variation): (TransactionData,i64) = match transaction_type {
            7 => {
                let transaction_data = DataTransactionType7 {
                    recipient_pk: signature::generate_public_key(data.as_slice()),
                    balance_variation: value,
                };
                (TransactionData::Type7(transaction_data), value + 1)
            }
            _ => {
                panic!("invalid transaction type");
            }
        };
        let signature = signature::new_signature(
            (timestamp.to_string()+&serde_json::to_string(&data).unwrap()).as_bytes(), 
            sk.as_slice());

        Transaction {
            timestamp,
            pk,
            data,
            balance_variation,
            signature,
        }
    }
}