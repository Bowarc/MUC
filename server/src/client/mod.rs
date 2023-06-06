mod filesystem;
mod handler;
pub use filesystem::*;
pub use handler::*;

const TARGET_TPS: f64 = 20.;

struct Client {
    socket: shared::networking::Socket<
        shared::networking::ClientMessage,
        shared::networking::ServerMessage,
    >,
    channel: crate::threading::Channel<Message>,
    account_state: Option<AccountState>,
    ip: std::net::SocketAddr,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

struct AccountState {
    id: uuid::Uuid,
    fs: filesystem::FileSystem,
}

// maybe add some to manage rights ?
#[derive(Debug, PartialEq)]
pub enum Message {
    LoginRequest { username: String, password: String },
    LoginResponse(Result<uuid::Uuid, crate::error::AccountLoginError>),
    LogoutRequest { id: uuid::Uuid },
    LogoutResponse(Result<(), crate::error::AccountLogoutError>),
}

impl Client {
    fn new(
        stream: std::net::TcpStream,
        channel: crate::threading::Channel<Message>,
        ip: std::net::SocketAddr,
        running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        // stream.set_nonblocking(false);
        Self {
            socket: shared::networking::Socket::<
                shared::networking::ClientMessage,
                shared::networking::ServerMessage,
            >::new(stream),
            channel,
            account_state: None,
            ip,

            running,
        }
    }

    fn run(&mut self) {
        let mut loop_helper = spin_sleep::LoopHelper::builder()
            .report_interval_s(0.5)
            .build_with_target_rate(TARGET_TPS);

        while self.running.load(std::sync::atomic::Ordering::Relaxed) {
            loop_helper.loop_start();

            self.handle_server_msg();

            self.handle_client_messages();

            loop_helper.loop_sleep();
        }

        self.socket.shutdown();

        if let Some(acc_state) = &self.account_state {
            self.channel
                .send(Message::LogoutRequest { id: acc_state.id })
                .unwrap();
        }
        debug!("Client thread for ({}) has exited", self.ip);
    }
    fn handle_server_msg(&mut self) {
        if let Ok(msg) = self.channel.try_recv() {
            match msg {
                Message::LoginResponse(result) => {
                    // send this to the client

                    if let Ok(id) = result {
                        self.account_state = Some(AccountState::new(id))
                    }
                    self.socket
                        .send(shared::networking::ServerMessage::LoginResponse(
                            result
                                // .map(|id| id.hyphenated().to_string())
                                .map_err(|e| format!("{e}")),
                        ))
                        .unwrap();
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

                    self.running
                        .store(false, std::sync::atomic::Ordering::Relaxed);

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
    }

    fn handle_client_messages(&mut self) {
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
                    if io_e.kind() == std::io::ErrorKind::ConnectionReset {
                        warn!("Client {ip} disconnected", ip = self.ip);
                        self.running
                            .store(false, std::sync::atomic::Ordering::Relaxed);
                        true
                    } else {
                        io_e.kind() == std::io::ErrorKind::WouldBlock
                    }
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
                    self.running
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    }
}

impl AccountState {
    pub fn new(user_id: uuid::Uuid) -> AccountState {
        AccountState {
            id: user_id,
            fs: filesystem::FileSystem::new(user_id),
        }
    }
}
