extern crate tungstenite;
extern crate zn_core;
use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use futures::{AsyncReadExt, SinkExt, StreamExt, TryFutureExt};
use log::info;
use std::env;
use std::sync::{Arc, Mutex};
use tungstenite::Message;
use zn_core::messages::{ClientMessage, ServerMessage};

mod xi;

pub async fn start_websocket_server() -> Result<(), std::io::Error> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        task::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let (xi_write_from_client, xi_read_to_client, _) = xi::start_xi_core();

    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let (mut ws_read, mut ws_write) = async_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred")
        .split();

    info!("New WebSocket connection: {}", addr);

    // Read WebSocket and send 2 XI
    let h1 = std::thread::spawn(move || loop {
        if let Some(msg) = async_std::task::block_on(ws_write.next()) {
            let msg: Message = msg.expect("ws_write.next() msg.expect");
            let msg_txt = msg.to_text().unwrap();
            info!("Raw msg.to_text() = {}", msg_txt);
            let js_msg = ClientMessage::from_json(msg.to_text().unwrap()).unwrap();
            info!("Sending message to XI: {:?}", js_msg.to_json());
            xi_write_from_client
                .0
                .send(js_msg.to_json().unwrap())
                .unwrap();
        }
    });

    // Read XI and send 2 WebSocket
    let h2 = std::thread::spawn(move || loop {
        if let Ok(msg) = xi_read_to_client.0.recv() {
            info!("Sending message to client {}", msg);
            let repr = ServerMessage::from_xi_json(&msg).unwrap();
            // send to client
            async_std::task::block_on(
                ws_read.send(tungstenite::Message::Text(repr.to_json().unwrap())),
            )
            .unwrap();
        }
    });
}
