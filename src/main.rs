use color_eyre::eyre::Result;

mod transport;
mod signature;
mod storage;
mod communication;

use storage::keyvalue;
use storage::merkle;

const DB_PATH: &str = "./storage.db";
#[tokio::main]
async fn main() -> Result<()> {

    println!("{}",communication::messages::Message::generate(1));


    // color_eyre::install()?;
    // let (node1, inc_1) = transport::new_node(4433).await?;
    // tokio::spawn(transport::listen(node1.clone(), inc_1, Box::new(|bytes| {
    //     let message = String::from_utf8(bytes.to_vec()).unwrap();
    //     println!("pppp {}", message);
    // })));
    // let (node2, inc_2) = transport::new_node(4434).await?;
    // tokio::spawn(transport::listen(node2.clone(), inc_2, Box::new(|bytes| {
    //     let message = String::from_utf8(bytes.to_vec()).unwrap();
    //     println!("pppp {}", message);
    // })));
    // let msg = communication::generate_msg(1).unwrap();
    // transport::client(&node2, node1.local_addr().to_string().as_str(), msg).await?;
    // node1.close();

    // let (sk, pk) = signature::new_pair();
    // let msg = b"hello world";
    // let signature = signature::new_signature(msg, &sk);
    // print!("{:?}",verify_signature(msg, &pk, &signature));

    // let trie = merkle::create_new();

    // //STORAGE ROCKSDB TEST

    // let database = keyvalue::open_db("./general-data");

    // database.put(b"key", b"value").unwrap();
    // database.get(b"key").unwrap();

    //TRANSPORT TEST
    

    
    // let (node3, inc_3) = transport::create_node(4435).await?;
    // let (node4, inc_4) = transport::create_node(4436).await?;
    // tokio::spawn(transport::callback(node2.clone(), inc_2));
    // tokio::spawn(transport::callback(node3.clone(), inc_3));
    // tokio::spawn(transport::callback(node4.clone(), inc_4));

    
    // transport::client(&node3, node2.local_addr().to_string().as_str(), "teste").await?;
    // transport::client(&node2, node3.local_addr().to_string().as_str(), "teste").await?;
    // transport::client(&node1, node4.local_addr().to_string().as_str(), "teste").await?;

    
    // node2.close();
    // node3.close();
    // node4.close();

    Ok(())
}