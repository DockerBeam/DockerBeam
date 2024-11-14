use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::StreamExt;
use futures_util::SinkExt;
use std::net::{SocketAddr, UdpSocket};
use stunclient::StunClient;


async fn get_public_ip_via_stun() -> Result<String, Box<dyn std::error::Error>> {
    // Use a well-known STUN server to determine the public IP address
    let (udp,sockaddr) = stunclient::just_give_me_the_udp_socket_and_its_external_address();
    Ok(sockaddr.ip().to_string())
}

pub async fn receive_docker_image() {
    eprintln!("aa");
    let addr = "0.0.0.0:8080".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    match get_public_ip_via_stun().await {
        Ok(ip) => println!("Your public IP address (via STUN): {}", ip),
        Err(e) => println!("Could not fetch public IP address via STUN: {}", e),
    }


    println!("WebSocket server listening on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.expect("Failed WebSocket handshake");
            println!("New WebSocket connection!");

            let (mut write, mut read) = ws_stream.split();
            while let Some(Ok(message)) = read.next().await {
                println!("Received: {:?}", message);
                if let Err(e) = write.send(message).await {
                    println!("Error sending message: {:?}", e);
                    break;
                }
            }
        });
    }
}