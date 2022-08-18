#![allow(dead_code)]

use std::{sync::{Arc, Mutex}, io::{Error, ErrorKind}};

use lazy_static::lazy_static;
use rocksdb;

use crate::DB_PATH;

lazy_static! {
    pub static ref DB_CONN: Arc<Mutex<rocksdb::DB>> = {
        let db = rocksdb::DB::open_default(DB_PATH).unwrap();
        Arc::new(Mutex::new(db))
    };
}

pub fn insert(key: &[u8], value: &[u8]) -> Result<(), std::io::Error> {
    match DB_CONN.lock().unwrap().put(key, value) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }
}

pub fn get(key: &[u8]) -> Result<Option<Vec<u8>>, std::io::Error> {
    match DB_CONN.lock().unwrap().get(key) {
        Ok(Some(value)) => Ok(Some(value)),
        Ok(None) => Ok(None),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }
}