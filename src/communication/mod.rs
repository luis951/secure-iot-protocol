pub mod messages;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{storage::keyvalue};

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
        let data = keyvalue::get(b"neighbors").unwrap().unwrap();
        let neighbors: Neighbors = serde_json::from_slice(data.as_slice()).unwrap();
        neighbors
    }

    fn add(src: String, pk: [u8; 33]) {
        let mut neighbors = Neighbors::restore();
        neighbors.neighbors.insert(src, Value::String(String::from_utf8(pk.to_vec()).unwrap()));
        keyvalue::insert(b"neighbors", serde_json::to_string(&neighbors).unwrap().as_bytes());
    }

    fn get(src: &str) -> Option<String> {
        let neigh_ref = Neighbors::restore();
        match neigh_ref.neighbors.get(src) {
            Some(v) => {
                let ret = v.as_str().unwrap().to_string();
                Some(ret)
            },
            None => None,
        }
    }
}