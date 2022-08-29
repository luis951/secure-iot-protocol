pub mod messages;
pub mod transactions;
pub mod responses;

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use serde_json::{Map, Value};

use crate::{storage::keyvalue};

#[derive(Serialize, Deserialize)]
pub struct Node {
    #[serde(with = "BigArray")]
    pk: [u8; 33],
    is_validator: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Neighbors {
    neighbors: Map<String, Value>,
}

impl Neighbors {
    fn new() {
        let neighbors = Neighbors {
            neighbors: Map::new()
        };
        keyvalue::insert(b"neighbors", serde_json::to_string(&neighbors).unwrap().as_bytes());
    }

    fn restore() -> Self {
        match keyvalue::get(b"neighbors").unwrap() {
            Some(data) => {
                let neighbors: Neighbors = serde_json::from_slice(data.as_slice()).unwrap();
                neighbors
            },
            None => {
                Neighbors::new();
                let data = keyvalue::get(b"neighbors").unwrap().unwrap();
                let neighbors: Neighbors = serde_json::from_slice(data.as_slice()).unwrap();
                neighbors
            }
        }
    }

    fn add(src: String, node: Node) {
        let mut neighbors = Neighbors::restore();
        neighbors.neighbors.insert(src, Value::String(serde_json::to_string(&node).unwrap()));
        keyvalue::insert(b"neighbors", serde_json::to_string(&neighbors).unwrap().as_bytes());
    }

    fn get(src: &str) -> Option<Node> {
        let neigh_ref = Neighbors::restore();
        match neigh_ref.neighbors.get(src) {
            Some(v) => {
                println!("{}", v);
                let ret = serde_json::from_str(&v.to_owned().as_str().unwrap().replace("//", "")).unwrap();
                Some(ret)
            },
            None => None,
        }
    }
}