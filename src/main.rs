use color_eyre::eyre::Result;
use std::str;

mod transport;
mod storage;

#[tokio::main]
async fn main() -> Result<()> {

    storage::open_trie_db();

    //STORAGE ROCKSDB TEST

    // let database = storage::open_db("./general-data");

    // database.put(b"key", b"value").unwrap();
    // database.get(b"key").unwrap();

    //TRANSPORT TEST
    
    // color_eyre::install()?;

    // let (node1, inc_1) = transport::create_node(4433).await?;
    // let (node2, inc_2) = transport::create_node(4434).await?;
    // let (node3, inc_3) = transport::create_node(4435).await?;
    // let (node4, inc_4) = transport::create_node(4436).await?;

    // tokio::spawn(transport::callback(node1.clone(), inc_1));
    // tokio::spawn(transport::callback(node2.clone(), inc_2));
    // tokio::spawn(transport::callback(node3.clone(), inc_3));
    // tokio::spawn(transport::callback(node4.clone(), inc_4));

    // transport::client(&node4, node1.local_addr().to_string().as_str(), "teste").await?;
    // transport::client(&node3, node2.local_addr().to_string().as_str(), "teste").await?;
    // transport::client(&node2, node3.local_addr().to_string().as_str(), "teste").await?;
    // transport::client(&node1, node4.local_addr().to_string().as_str(), "teste").await?;

    // node1.close();
    // node2.close();
    // node3.close();
    // node4.close();

    Ok(())
}