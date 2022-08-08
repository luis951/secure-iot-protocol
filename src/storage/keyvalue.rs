#![allow(dead_code)]
#![allow(unused_variables)]

use rocksdb;

pub fn open_db(path: &str) -> rocksdb::DBWithThreadMode<rocksdb::SingleThreaded>{
    let database = rocksdb::DB::open_default(path).unwrap();
    return database;
}