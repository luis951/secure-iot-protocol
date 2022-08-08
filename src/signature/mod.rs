#![allow(dead_code)]
#![allow(unused_variables)]

use secp256k1::rand::{thread_rng};
use secp256k1::{Message, Secp256k1, SecretKey, PublicKey, ecdsa::Signature, Error};
use secp256k1::hashes::sha256;

pub fn new_pair() -> ([u8; 32], [u8; 33]){
    let secp = Secp256k1::new();
    let (sk, pk) = secp.generate_keypair(&mut thread_rng());
    return (sk.secret_bytes(), pk.serialize());
}

pub fn generate_public_key(secret_key: &[u8]) -> secp256k1::PublicKey{
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(secret_key).expect("32 bytes, within curve order");
    PublicKey::from_secret_key(&secp, &sk)
}

pub fn new_signature(msg: &[u8], secret_key: &[u8]) -> [u8; 64]{
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(secret_key).expect("32 bytes, within curve order");
    let parsed_msg = Message::from_hashed_data::<sha256::Hash>(msg);

    secp.sign_ecdsa(&parsed_msg, &sk).serialize_compact()
}

pub fn verify_signature(msg: &[u8], public_key: &[u8], signaure: &[u8]) -> Result<(), Error>{
    let secp = Secp256k1::new();

    let parsed_msg = Message::from_hashed_data::<sha256::Hash>(msg);
    let pk = PublicKey::from_slice(public_key).expect("32 bytes, within curve order");
    let sig = Signature::from_compact(signaure).expect("64 bytes");
    secp.verify_ecdsa(&parsed_msg, &sig, &pk)
}

