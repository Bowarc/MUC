pub mod handler;

#[derive(Debug, PartialEq)]
pub enum Message {}

pub struct Server {
    server_state: ServerState,
    channel: shared::threading::Channel<Message>,
    account_state: Option<AccountState>,
}

enum ServerState {
    Offline,
    Connected {
        ip: std::net::SocketAddr,
        socket: shared::networking::Socket<
            shared::networking::ServerMessage,
            shared::networking::ClientMessage,
        >,
    },
}

struct AccountState {
    id: uuid::Uuid,
    fs: shared::filesystem::FileScan,
}

impl Server {
    pub fn new(channel: shared::threading::Channel<Message>) -> Self {
        Self {
            server_state: ServerState::Offline,
            channel,
            account_state: None,
        }
    }

    pub fn run(&mut self) {
        loop {
            match &mut self.server_state {
                ServerState::Connected { .. } => {
                    self.handle_client_messages();

                    self.handle_server_messages();
                }
                ServerState::Offline => self.connect_to_server(),
            }
        }
    }

    fn connect_to_server(&mut self) {}

    fn handle_client_messages(&mut self) {
        if let Ok(msg) = self.channel.try_recv() {
            match msg {};
        }
    }
    fn handle_server_messages(&mut self) {
        let (socket, ip) = if let ServerState::Connected { ip, socket } = &mut self.server_state {
            (socket, ip)
        } else {
            return;
        };

        match socket.recv() {
            Ok(msg) => match msg {
                shared::networking::ServerMessage::Text(_) => {
                    //
                }
                shared::networking::ServerMessage::LoginResponse(_) => {
                    //
                }
                shared::networking::ServerMessage::LogoutResponse(_) => {
                    //
                }
            },
            Err(e) => {
                if if let shared::networking::SocketError::StreamRead(ref io_e) = e {
                    if io_e.kind() == std::io::ErrorKind::ConnectionReset {
                        warn!("Client {ip} disconnected", ip = ip);

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
                }
            }
        }
    }
}
