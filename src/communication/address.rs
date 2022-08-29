#[derive(Serialize, Deserialize)]
struct Address {
    pk: [u8; 33],
    last_transaction_tree_root: Vec<u8>,
    last_transaction_tree_position: Vec<u8>,
    balance: u64,
}

#[derive(Serialize, Deserialize)]
struct AllAddresses {
    addresses: Vec<Address>,
}