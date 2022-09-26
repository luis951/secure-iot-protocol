use std::io::{Error, ErrorKind};

use sled;

pub fn new() -> Result<sled::Db, Error> {
    match sled::open("test.db") {
        Ok(db) => Ok(db),
        Err(e) => Err(Error::new(ErrorKind::Other,e)),
    }
}

pub fn insert(db: &sled::Db, key: &str, value: &str) -> Result<(), Error> {
    match db.insert(key, value) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorKind::Other,e)),
    }
}

pub fn get(db: &sled::Db, key: &str) -> Result<String, Error> {
    match db.get(key) {
        Ok(Some(value)) => Ok(String::from_utf8(value.to_vec()).unwrap()),
        Ok(None) => Err(Error::new(ErrorKind::Other,"Key not found")),
        Err(e) => Err(Error::new(ErrorKind::Other,e)),
    }
}