#[macro_use]
extern crate log;

use serde::{Deserialize, Serialize};

pub mod client;
pub mod id;
pub mod logger;
pub mod server;

// type PublicKey = String;

// #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
// pub enum Message {
//     // Connected,
//     // OkConencted,
//     // Disconnect,
//     // OkDisconnect,
//     Client(ClientMessage),
//     Server(ServerMessage),
//     None,
//     // Text(String),
// }
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ClientMessage {
    ConnectionRequest(String), // public key
    DisconnectionRequest,
    Text(String),
    None,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ServerMessage {
    Connected(String),
    AlreadyConnected,
    Disconnected,
    None,
    // DisconnectOrdern, //(reason)?
}

// pub struct Message {
//     encrypted: bool,
//     content: MessageContent,
// }
