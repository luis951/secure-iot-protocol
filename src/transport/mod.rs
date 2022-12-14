#![allow(dead_code)]
#![allow(unused_variables)]

use async_once::AsyncOnce;
use async_recursion::async_recursion;
use color_eyre::eyre::Result;
use bytes::Bytes;
use lazy_static::lazy_static;
use qp2p::{Config, Endpoint, IncomingConnections};
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration, sync::{Arc},
};
use tokio::sync::Mutex;

use crate::{PORT_NUMBER, communication::{messages::{Packet}, responses::Response}};

lazy_static! {
    // QUIC_CONN.1 = Endpoint, QUIC_CONN.2 = IncomingConnections

    pub static ref QUIC_CONN: AsyncOnce<(Arc<Mutex<Endpoint>>, Arc<Mutex<IncomingConnections>>)> = 
        AsyncOnce::new(async{
            let (node, incoming, _contact) = Endpoint::new_peer(
                SocketAddr::from((Ipv4Addr::UNSPECIFIED, PORT_NUMBER.parse().unwrap())),
                &[],
                Config {
                    idle_timeout: Duration::from_secs(60 * 60).into(), // 1 hour idle timeout.
                    ..Default::default()
                },
            ).await.unwrap();
            (Arc::new(Mutex::new(node)), Arc::new(Mutex::new(incoming)))
        });
    

    pub static ref LOCAL_ADDR: AsyncOnce<std::string::String> = AsyncOnce::new(async{
        QUIC_CONN.get().await.0.lock().await.public_addr().to_string()
    });
}

pub async fn listen() -> Result<()> {
    
    // loop over incoming connections
    while let Some((connection, mut incoming_messages)) = 
    QUIC_CONN.get().await.1.lock().await.next().await {
        let src = connection.remote_address();

        // loop over incoming messages
        while let Some(bytes) = incoming_messages.next().await? {
            match callback(&bytes.to_vec(), 
            src.to_string()).await {
                Some(response) => {
                    connection.send(Bytes::from(response)).await?;
                }
                None => {
                    println!("Error: No response");
                }
            }
        }
    }

    Ok(())
}

pub async fn send(addr: String, msg: String) -> Result<()> {

    let peer: SocketAddr = addr
            .parse()
            .expect("Invalid SocketAddr.  Use the form 127.0.0.1:1234");
        let msg = Bytes::from(msg);
        println!("Enviando para {:?} --> {:?}\n", peer, msg);
        let (conn, mut incoming) = QUIC_CONN.get().await.0.lock().await.connect_to(&peer).await?;
        conn.send(msg.clone()).await?;
        // `Endpoint` no longer having `connection_pool` to hold established connection.
        // Which means the connection get closed immediately when it reaches end of life span.
        // And causes the receiver side a sending error when reply via the in-coming connection.
        // Hence here have to listen for the reply to avoid such error
        let reply = incoming.next().await?.unwrap();
        callback(&reply.to_vec(), peer.to_string()).await;

    println!("Envio finalizado");
    Ok(())
}

#[async_recursion]
pub async fn callback(bytes: &Vec<u8>, src: String) -> Option<String> {

    println!("Recebido de {:?} --> {:?}\n", src, std::str::from_utf8(bytes).unwrap());
    let request: Packet = serde_json::from_slice(bytes).unwrap();

            let response: Option<Packet>;
            match request {
                Packet::Message(msg) => {
                    response = Some(match msg.execute(src.clone()).await {
                        Ok(response) => {
                            println!("Enviando resposta para {:?} --> {:?}\n", src.clone(), serde_json::to_string(&response).unwrap());
                            Packet::Response(response)
                        },
                        Err(err) => {
                            println!("Erro: {:?}", err);
                            Packet::Response(Response::generate(500).unwrap())
                        }});
                },
                Packet::Response(res) => {
                    res.execute(src.clone()).await.unwrap();
                    response = None
                }
            } 

            match response {
                Some(resp) => {
                    let response_bytes = serde_json::to_string(&resp).unwrap();
                    Some(response_bytes)
                }
                None => None,
            }
}