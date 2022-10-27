#![allow(dead_code)]
#![allow(unused_variables)]

use cita_trie::{MemoryDB, Trie};
use lazy_static::lazy_static;
use tokio::sync::RwLock;
use std::sync::Arc;
use hasher::HasherKeccak;
use cita_trie::PatriciaTrie;

lazy_static! {

    static ref LOCAL_BLOCK: RwLock<PatriciaTrie<MemoryDB, HasherKeccak>> = {
        let db = MemoryDB::new(true);
        let memdb = Arc::new(db);
        let hasher = Arc::new(HasherKeccak::new());

        let trie = PatriciaTrie::new(Arc::clone(&memdb), Arc::clone(&hasher));
    
        RwLock::new(trie)
    };

    pub static ref LOCAL_BLOCK_SIZE: RwLock<u64> = RwLock::new(0);
}

pub async fn get_trie_data_as_vector() -> Vec<u8> {
    let i: Vec<u8> = (*LOCAL_BLOCK.read().await.db).serialize();
    i
}

pub async fn reset_local_trie(){
    let db = MemoryDB::new(true);
    let memdb = Arc::new(db);
    let hasher = Arc::new(HasherKeccak::new());

    let trie = PatriciaTrie::new(Arc::clone(&memdb), Arc::clone(&hasher));
    *LOCAL_BLOCK.write().await = trie;
}

pub async fn get_local_trie() -> (Vec<u8>, Vec<u8>) {

    let root = LOCAL_BLOCK.write().await.root().unwrap();
    let data = LOCAL_BLOCK.read().await.db.serialize();
    (root,data)
}

pub async fn create_trie_from_str(body: Vec<u8>, root: Vec<u8>) {
    let db = MemoryDB::new((true, body));
    let memdb = Arc::new(db);
    let hasher = Arc::new(HasherKeccak::new());

    let trie = PatriciaTrie::from(Arc::clone(&memdb), Arc::clone(&hasher), &root).unwrap();
    
    *LOCAL_BLOCK.write().await = trie;

}

pub fn create_evaluation_trie(str: Vec<u8>, root: Vec<u8>) -> PatriciaTrie<MemoryDB, HasherKeccak> {
    let db = MemoryDB::new((true, str));
    let memdb = Arc::new(db);
    let hasher = Arc::new(HasherKeccak::new());

    PatriciaTrie::from(Arc::clone(&memdb), Arc::clone(&hasher), &root).unwrap()
}

pub async fn insert(key: &[u8], value: &[u8]) {
    let mut trie = LOCAL_BLOCK.write().await;
    trie.insert(key.to_vec(), value.to_vec()).unwrap();
}

pub async fn get(key: Vec<u8>) -> Option<Vec<u8>> {
    let trie = LOCAL_BLOCK.read().await;
    trie.get(key.as_slice()).unwrap()
}

pub async fn get_all() -> Vec<(Vec<u8>, Vec<u8>)> {
    let trie = LOCAL_BLOCK.read().await;
    trie.iter().collect()
}