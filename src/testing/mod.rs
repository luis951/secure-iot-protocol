mod transport;
mod storage;
use crate::{signature, communication};

pub async fn transport_tests(){
    println!("\n\n--------------------- TRANSPORT MODULE TESTS ---------------------");

    let (server1, incoming1) = transport::test_server("10001".to_string()).await.unwrap();
    let (server2, incoming2) = transport::test_server("10002".to_string()).await.unwrap();

    let (client1, c_incoming1) = transport::test_server("10003".to_string()).await.unwrap();
    let (client2, c_incoming2) = transport::test_server("10004".to_string()).await.unwrap();

    tokio::spawn(
        transport::test_listen(incoming1)
    );
    tokio::spawn(
        transport::test_listen(incoming2)
    );

    println!("Server 1 listening on port 10001");
    println!("Server 2 listening on port 10002");

    println!("Transport module test #1: OK\n");

    match transport::test_send(client1.clone(), server1.public_addr().to_string(), "Hello from client 1".to_string()).await {
        Ok(_) => {
            println!("Transport module test #2: OK\n");},
        Err(e) => {
            panic!("Transport module test #2: FAILED\nERROR: {}", e);
        },
    }

    for t in 0..10 {
        match transport::test_send(client1.clone(), server1.public_addr().to_string(), format!("Message # {} from client 1", t)).await {
            Ok(_) => {},
            Err(e) => {
                panic!("Transport module test #3: FAILED\nERROR: {}", e);
            },
        }
    }
    println!("Transport module test #3: OK\n");


    for t in 0..10 {
        match transport::test_send(if t%2==0 {client1.clone()} else {client2.clone()}, server1.public_addr().to_string(), format!("Message # {} from client {}", t, t%2+1)).await {
            Ok(_) => {},
            Err(e) => {
                panic!("Transport module test #4: FAILED\nERROR: {}", e);
            },
        }
        match transport::test_send(if t%2==0 {client1.clone()} else {client2.clone()}, server2.public_addr().to_string(), format!("Message # {} from client {}", t, t%2+1)).await {
            Ok(_) => {},
            Err(e) => {
                panic!("Transport module test #4: FAILED\nERROR: {}", e);
            },
        }
    }
    println!("Transport module test #4: OK\n");

}

pub fn signature_tests(){

    println!("\n\n--------------------- SIGNATURE MODULE TESTS ---------------------");

    println!("Generating new keypair...");
    
    let (sk, pk) = signature::new_pair();

    println!("Signature module test #1: OK\n");

    let msg = "Hello world";

    println!("Generating signature to string \"Hello World\"...");

    let sig = signature::new_signature(msg.as_bytes(), &sk);

    println!("Signature module test #2: OK\n");

    println!("Verifying signature...");

    let valid = signature::verify_signature(msg.as_bytes(), &pk, &sig);
    match valid {
        Ok(_) => {
            println!("Signature module test #3: OK\n");
        },
        Err(e) => {
            panic!("Signature module test #3: FAILED\nERROR: {}", e);
        },
    }

    println!("Generating and verifying invalid signature...");

    let new_sig = signature::new_signature(b"Random data", &sk);
    let invalid = signature::verify_signature(msg.as_bytes(), &pk, &new_sig);
    match invalid {
        Ok(_) => {
            panic!("Signature module test #4: FAILED\nERROR: Signature verification should have failed");
        },
        Err(e) => {
            println!("Signature module test #4: OK\n");
        },
    }
}

pub fn storage_tests(){
    println!("\n\n--------------------- STORAGE MODULE TESTS ---------------------");

    let db = match storage::new(){
        Ok(db) => {
            println!("Storage module test #1: OK\n");
            db},
        Err(e) => panic!("Storage module test #1: FAILED\nERROR: {}", e),
    };

    let mut data = "Hello world".to_string();
    data += &checksum(data.as_bytes()).to_string();

    match storage::insert(&db, "key1", &data){
        Ok(_) => {
            println!("Storage module test #2: OK\n");
        },
        Err(e) => panic!("Storage module test #2: FAILED\nERROR: {}", e),
    }

    match storage::get(&db, "key1"){
        Ok(d) => {
            if d == data {
                println!("Storage module test #3: OK\n");
            } else {
                panic!("Storage module test #3: FAILED\nERROR: Data mismatch");
            }
        },
        Err(e) => panic!("Storage module test #3: FAILED\nERROR: {}", e),
    }

    match storage::get(&db, "key2"){
        Ok(_) => {
            panic!("Storage module test #4: FAILED\nERROR: Key should not exist");
        },
        Err(e) => {
            println!("Storage module test #4: OK\n");
        },
    }

}

pub fn comm_test(){
    let msg = communication::messages::Message::generate(1);
    let serialized_msg = serde_json::to_string(&msg).unwrap();
    let deserialized_msg: communication::messages::Message = serde_json::from_str(&serialized_msg).unwrap();
}

fn checksum(data: &[u8]) -> u32 {
    let mut s:u32 = 0;
    for &byte in data {
        s = s.wrapping_add(byte as u32);
    }
    s
}