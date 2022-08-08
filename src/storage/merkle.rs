#![allow(dead_code)]
#![allow(unused_variables)]

use cita_trie::MemoryDB;
use std::sync::{Arc};
use hasher::{HasherKeccak};
use cita_trie::{PatriciaTrie};


pub fn create_new() -> PatriciaTrie<MemoryDB, HasherKeccak>{
    let db = MemoryDB::new(true);
    let memdb = Arc::new(db);
    let hasher = Arc::new(HasherKeccak::new());

    let trie = PatriciaTrie::new(Arc::clone(&memdb), Arc::clone(&hasher));
    return trie;
}

pub fn create_tree_from_str(str: String, root: Vec<u8>) -> PatriciaTrie<MemoryDB, HasherKeccak>{
    let db = MemoryDB::new((true, str));
    let memdb = Arc::new(db);
    let hasher = Arc::new(HasherKeccak::new());

    let trie = PatriciaTrie::from(Arc::clone(&memdb), Arc::clone(&hasher), &root).unwrap();
    return trie;
}