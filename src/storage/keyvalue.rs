#![allow(dead_code)]

use std::{sync::{Arc, Mutex}, io::{Error, ErrorKind}};

use lazy_static::lazy_static;
use sled;

use crate::DB_PATH;

lazy_static! {
    static ref DB_CONN: Arc<Mutex<sled::Db>> = {
        let db = sled::open(DB_PATH).unwrap();
        Arc::new(Mutex::new(db))
    };
}

pub fn insert(key: &[u8], value: &[u8]) -> Result<(), std::io::Error> {
    match DB_CONN.lock().unwrap().insert(key, value) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }
}

pub fn get(key: &[u8]) -> Result<Option<Vec<u8>>, std::io::Error> {
    match DB_CONN.lock().unwrap().get(key) {
        Ok(Some(value)) => Ok(Some(value.to_vec())),
        Ok(None) => Ok(None),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }
}