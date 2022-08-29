use std::io::Error;

use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize)]
struct Type1Data {
    #[serde(with = "BigArray")]
    pk: [u8; 33],
}

impl Type1Data {
    pub fn execute(&self, src: String) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
enum Data {
    ResponseType1(Type1Data), // Added to node list, data is public key
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    timestamp: i64,
    data: Data,
    #[serde(with = "BigArray")]
    signature: [u8; 64]
}
