mod files;
mod handler;
pub use files::*;
pub use handler::*;

struct Client {
    socket: shared::networking::Socket<
        shared::networking::ClientMessage,
        shared::networking::ServerMessage,
    >,
    channel: crate::threading::Channel<Message>,
    account_state: Option<uuid::Uuid>,
}

#[derive(Debug, PartialEq)]
pub enum Message {
    LoginRequest { username: String, password: String },
    LoginResponse(Result<uuid::Uuid, crate::error::AccountLoginError>),
    LogoutRequest { id: uuid::Uuid },
    LogoutResponse(Result<(), crate::error::AccountLogoutError>),
}

impl Client {
    fn new(stream: std::net::TcpStream, channel: crate::threading::Channel<Message>) -> Self {
        // stream.set_nonblocking(false);
        Self {
            socket: shared::networking::Socket::<
                shared::networking::ClientMessage,
                shared::networking::ServerMessage,
            >::new(stream),
            channel,
            account_state: None,
        }
    }

    fn run(&mut self) {
        loop {
            // debug!("Client loop");
            // think about handleing quit events too
            //handle_server_msg
            if let Ok(msg) = self.channel.try_recv() {
                match msg {
                    Message::LoginResponse(result) => {
                        // send this to the client

                        if let Ok(id) = result {
                            self.account_state = Some(id)
                        }
                        self.socket
                            .send(shared::networking::ServerMessage::LoginResponse(
                                result
                                    .map(|id| id.hyphenated().to_string())
                                    .map_err(|e| format!("{e}")),
                            ))
                            .unwrap();

                        // match result {
                        //     Ok(id) => {
                        //         self.account_state = Some(id);
                        //         shared::networking::ServerMessage::Text(format!(
                        //             "[text] You're now connected with account id: {id}",
                        //         ))
                        //     }
                        //     Err(e) => shared::networking::ServerMessage::Text(format!(
                        //         "[text] Your connection request was rejected, reason: {e}",
                        //     )),
                        // }
                    }
                    Message::LogoutResponse(result) => {
                        if result.is_ok() {
                            self.account_state = None
                        }

                        self.socket
                            .send(shared::networking::ServerMessage::LogoutResponse(
                                result.map_err(|e| format!("{e}")),
                            ))
                            .unwrap();

                        break;

                        // match result {
                        //     Ok(_) => {
                        //         self.account_state = None;
                        //         shared::networking::ServerMessage::Text(
                        //             "[text] Successfully logged out".to_string(),
                        //         )
                        //     }
                        //     Err(e) => shared::networking::ServerMessage::Text(format!(
                        //         "[text] Logout request failled, reason: {e}",
                        //     )),
                        // }
                    }

                    Message::LoginRequest { .. } | Message::LogoutRequest { .. } => unreachable!(),
                };
            }

            //handle_client_messages
            match self.socket.recv() {
                Ok(msg) => match msg {
                    shared::networking::ClientMessage::Text(txt) => {
                        debug!("Client {} sent {}", self.socket.remote_addr(), txt)
                    }
                    shared::networking::ClientMessage::LoginRequest { username, password } => self
                        .channel
                        .send(Message::LoginRequest { username, password })
                        .unwrap(),
                    shared::networking::ClientMessage::LogoutRequest { id } => {
                        self.channel.send(Message::LogoutRequest { id }).unwrap();
                    }
                },
                Err(e) => {
                    if if let shared::networking::SocketError::StreamRead(ref io_e) = e {
                        io_e.kind() == std::io::ErrorKind::WouldBlock
                    } else {
                        // matches!(e, shared::networking::SocketError::WouldBlock)
                        false
                    } {
                        // Not critical error
                        // warn!("Would block");
                    } else {
                        error!(
                            "Error while listening client {}, aborting: {e}",
                            self.socket.remote_addr()
                        );
                        break;
                    }
                }
            }
        }
        self.socket.shutdown();
        debug!("Client has exited");

        if let Some(id) = self.account_state {
            self.channel.send(Message::LogoutRequest { id }).unwrap();
        }
    }
    fn handle_server_msg(&mut self) {}

    fn handle_client_messages(&mut self) {}
}
