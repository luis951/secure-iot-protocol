#![allow(dead_code)]
#![allow(unused_variables)]

use color_eyre::eyre::Result;
use bytes::Bytes;
use qp2p::{Config, Endpoint, IncomingConnections};
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

pub async fn create_node(port: u16) -> Result<(Endpoint, IncomingConnections)>{

    // create an endpoint for us to listen on and send from.
    let (node, incoming_conns, _contact) = Endpoint::new_peer(
        SocketAddr::from((Ipv4Addr::LOCALHOST, port)),
        &[],
        Config {
            idle_timeout: Duration::from_secs(60 * 60).into(), // 1 hour idle timeout.
            ..Default::default()
        },
    ).await?;

    println!("\n---");
    println!("Listening on: {:?}", node.public_addr());
    println!("---\n");

    Ok((node, incoming_conns))
}

pub async fn callback(node: Endpoint, mut incoming_conns: IncomingConnections) -> Result<()> {
    
    // loop over incoming connections
    while let Some((connection, mut incoming_messages)) = incoming_conns.next().await {
        let src = connection.remote_address();

        // loop over incoming messages
        while let Some(bytes) = incoming_messages.next().await? {
            println!("{:?} received from {:?} --> {:?}", node.local_addr(), src, bytes);
            let response = Bytes::from("200");
            connection.send(response.clone()).await?;
            println!("{:?} replied to {:?} --> {:?}", node.local_addr(), src, response);
            println!();
        }
    }

    Ok(())
}

pub async fn client(node: &Endpoint, addr: &str, msg: &'static str) -> Result<()> {

    let peer: SocketAddr = addr
            .parse()
            .expect("Invalid SocketAddr.  Use the form 127.0.0.1:1234");
        let msg = Bytes::from(msg);
        println!("Sending to {:?} --> {:?}\n", peer, msg);
        let (conn, mut incoming) = node.connect_to(&peer).await?;
        conn.send(msg.clone()).await?;
        // `Endpoint` no longer having `connection_pool` to hold established connection.
        // Which means the connection get closed immediately when it reaches end of life span.
        // And causes the receiver side a sending error when reply via the in-coming connection.
        // Hence here have to listen for the reply to avoid such error
        let reply = incoming.next().await?.unwrap();
        println!("Received from {:?} --> {:?}", peer, reply);

    println!("Done sending");
    Ok(())
}