use std::{net::{SocketAddr, Ipv4Addr}, time::Duration, io::{Error, ErrorKind}};
use bytes::Bytes;
use color_eyre::Result;
use qp2p::{Endpoint, IncomingConnections, Config};

pub async fn test_server(port_number: String) -> Result<(Endpoint, IncomingConnections), Error> {
    match Endpoint::new_peer(
        SocketAddr::from((Ipv4Addr::LOCALHOST, port_number.parse().unwrap())),
        &[],
        Config {
            idle_timeout: Duration::from_secs(60 * 60).into(), // 1 hour idle timeout.
            ..Default::default()
        },
    ).await {
        Ok((node, incoming, _contact)) => Ok((node, incoming)),
        Err(e) => Err(Error::new(ErrorKind::AddrNotAvailable, e)),
    }
}

pub async fn test_listen(mut incoming: IncomingConnections) -> Result<()> {
    while let Some((connection, mut incoming_messages)) = incoming.next().await {
        let src = connection.remote_address();

        // loop over incoming messages
        while let Some(bytes) = incoming_messages.next().await? {
            println!("Received: {:?} from {:?}", String::from_utf8(bytes.to_vec()), src.to_string());
            connection.send(Bytes::from("Ok")).await?;
        }
    }

    Ok(())
}

pub async fn test_send(node: Endpoint, addr: String, msg: String) -> Result<()> {
    let peer: SocketAddr = addr
            .parse()
            .expect("Invalid SocketAddr.  Use the form 127.0.0.1:1234");

    let (connection, mut incoming) = node.connect_to(&addr.parse().unwrap()).await?;
    connection.send(Bytes::from(msg)).await?;
    let reply = incoming.next().await?.unwrap();
    println!("Received from {:?} --> {:?}", peer, reply);
    Ok(())
}