use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize, Clone)]
pub struct AddressFormat{
    #[serde(with = "BigArray")]
    pub address: [u8; 33],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Address {
    #[serde(with = "BigArray")]
    pub last_transaction: [u8; 64],
    pub balance: i64,
    pub linked_addresses: Vec<AddressFormat>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AddressesState {
    pub state: Map<String, Value>
}

impl AddressesState {
    pub fn new() -> Self {
        AddressesState {
            state: Map::new()
        }
    }
}