use rocksdb;
use std::sync::Arc;
use hasher::{Hasher, HasherKeccak};
use cita_trie::MemoryDB;
use cita_trie::{PatriciaTrie, Trie};
use std::str;

pub fn open_db(path: &str) -> rocksdb::DBWithThreadMode<rocksdb::SingleThreaded>{
    let database = rocksdb::DB::open_default(path).unwrap();
    return database;
}

pub fn open_trie_db() -> PatriciaTrie<MemoryDB, HasherKeccak>{
    let memdb = Arc::new(MemoryDB::new(true));
    let hasher = Arc::new(HasherKeccak::new());

    let key = "test-key".as_bytes();
    let value = "test-value".as_bytes();

    let root = {
        let mut trie = PatriciaTrie::new(Arc::clone(&memdb), Arc::clone(&hasher));
        trie.insert(key.to_vec(), value.to_vec()).unwrap();

        let v = trie.get(key).unwrap();
        assert_eq!(Some(value.to_vec()), v);
        trie.root().unwrap()
    };

    let mut trie = PatriciaTrie::from(Arc::clone(&memdb), Arc::clone(&hasher), &root).unwrap();
    let proof = trie.get_proof("test-key".as_bytes()).unwrap();
    println!("{:?}",str::from_utf8(&trie.verify_proof(root, "test-key".as_bytes(), proof).unwrap().unwrap()));
    return trie;

}