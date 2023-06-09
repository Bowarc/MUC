pub mod handler;

#[derive(Debug, PartialEq)]
pub enum Message {
    LoginRequest { username: String, password: String },
    LoginResponse(Result<uuid::Uuid, String>),

    LogoutRequest { id: uuid::Uuid },
    LogoutResponse(Result<(), String>),

    FileScanUpdate(shared::filesystem::FileScan),
    ChangeDirectory(String),
}

pub struct Server {
    server_state: ServerState,
    channel: shared::threading::Channel<Message>,
}

enum ServerState {
    Offline,
    Connected {
        socket: shared::networking::Socket<
            shared::networking::ServerMessage,
            shared::networking::ClientMessage,
        >,
    },
}

pub struct AccountState {
    pub id: uuid::Uuid,
    pub fs: Option<shared::filesystem::FileScan>,
}

impl Server {
    pub fn new(channel: shared::threading::Channel<Message>) -> Self {
        Self {
            server_state: ServerState::Offline,
            channel,
        }
    }

    pub fn run(&mut self) {
        loop {
            match &self.server_state {
                ServerState::Connected { .. } => {
                    self.handle_client_messages();

                    self.handle_server_messages();
                }
                ServerState::Offline => self.connect_to_server(),
            }
        }
    }

    fn connect_to_server(&mut self) {
        let stream = std::net::TcpStream::connect(shared::networking::DEFAULT_ADDRESS).unwrap();
        stream.set_nonblocking(true).unwrap();
        let socket = shared::networking::Socket::<
            shared::networking::ServerMessage,
            shared::networking::ClientMessage,
        >::new(stream);

        self.server_state = ServerState::Connected { socket };
        debug!("Connected");
    }

    fn handle_client_messages(&mut self) {
        let socket = if let ServerState::Connected { socket } = &mut self.server_state {
            socket
        } else {
            debug!("Nosocket");
            return;
        };

        if let Ok(msg) = self.channel.try_recv() {
            debug!("{msg:?}");
            match msg {
                Message::LoginRequest { username, password } => socket
                    .send(shared::networking::ClientMessage::LoginRequest { username, password })
                    .unwrap(),
                Message::LoginResponse(resp) => self
                    .channel
                    .send(crate::server::Message::LoginResponse(resp))
                    .unwrap(),
                Message::LogoutRequest { id } => socket
                    .send(shared::networking::ClientMessage::LogoutRequest { id })
                    .unwrap(),
                Message::LogoutResponse(resp) => self
                    .channel
                    .send(crate::server::Message::LogoutResponse(resp))
                    .unwrap(),
                Message::FileScanUpdate(scan) => self
                    .channel
                    .send(crate::server::Message::FileScanUpdate(scan))
                    .unwrap(),
                Message::ChangeDirectory(new_dir) => socket
                    .send(shared::networking::ClientMessage::ChangeDirectory(new_dir))
                    .unwrap(),
            };
        }
    }
    fn handle_server_messages(&mut self) {
        let socket = if let ServerState::Connected { socket } = &mut self.server_state {
            socket
        } else {
            debug!("Nosocket");
            return;
        };

        match socket.recv() {
            Ok(msg) => {
                debug!("Received {msg:?}");
                match msg {
                    shared::networking::ServerMessage::Text(t) => {
                        debug!("Server sent: '{t}'")
                    }
                    shared::networking::ServerMessage::LoginResponse(resp) => {
                        self.channel.send(Message::LoginResponse(resp)).unwrap()
                    }
                    shared::networking::ServerMessage::LogoutResponse(resp) => {
                        self.channel.send(Message::LogoutResponse(resp)).unwrap()
                    }
                    shared::networking::ServerMessage::FileScanUpdate(scan) => {
                        self.channel.send(Message::FileScanUpdate(scan)).unwrap()
                    }
                }
            }
            Err(e) => {
                if if let shared::networking::SocketError::StreamRead(ref io_e) = e {
                    if io_e.kind() == std::io::ErrorKind::ConnectionReset {
                        warn!("Server disconnected");

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
                    error!("Error while listening server, aborting: {e}",);
                    self.server_state = ServerState::Offline
                }
            }
        }
    }
}
