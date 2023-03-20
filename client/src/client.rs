use rsa::pkcs8::DecodePublicKey;
use rsa::pkcs8::EncodePublicKey;
use rsa::PublicKey;

pub enum ClientState {
    Disconnected,
    Connected(rsa::RsaPublicKey),
    WaitingForServerPublicKey,
}

pub struct Client {
    addr: std::net::SocketAddr,
    socket: laminar::Socket,
    private_key: rsa::RsaPrivateKey,
    // server_public_key: Option<rsa::RsaPublicKey>,
    state: ClientState,
}

impl Client {
    pub fn new(base_addr: std::net::SocketAddr) -> Self {
        let mut addr = base_addr;

        let socket = loop {
            let s = laminar::Socket::bind(addr);
            if let Ok(socket) = s {
                break socket;
            } else {
                addr.set_port(addr.port() + 1);
                continue;
            }
        };

        Self {
            addr,
            socket,
            private_key: rsa::RsaPrivateKey::new(&mut rand::thread_rng(), 2048).unwrap(),
            // server_public_key: None,
            state: ClientState::Disconnected,
        }
    }
    fn verify_client_message(&self, message: &shared::ClientMessage) -> Result<(), String> {
        // if we're disconnected and the message is not a connection request
        if matches!(self.state, ClientState::Disconnected)
            && !matches!(message, shared::ClientMessage::ConnectionRequest(_))
        {
            return Err(format!(
                "Client is disconnected and the message is not a connection request: {message:?}",
            ));
        }

        // if we're connected and we're asking to be connected
        if matches!(self.state, ClientState::Connected(_))
            && matches!(message, shared::ClientMessage::ConnectionRequest(_))
        {
            return Err("Client id connected and we're asking to be connected".to_string());
        }

        // if we're waiting for the server's public key and we're re-asking for it
        if matches!(self.state, ClientState::WaitingForServerPublicKey)
            && matches!(message, shared::ClientMessage::ConnectionRequest(_))
        {
            return Err(
                "Client is waiting for the server's public key and is re-asking for it".to_string(),
            );
        }

        Ok(())
    }

    pub fn send(&mut self, message: shared::ClientMessage) {
        if let Some(e) = self.verify_client_message(&message).err() {
            error!("{e}");
            return;
        }

        let mut message_bytes = bincode::serialize(&message).unwrap();

        // If the server has sent us its public key, use it to encrypt the data we want to send
        if let ClientState::Connected(server_public_key) = &self.state {
            message_bytes = server_public_key
                .encrypt(
                    &mut rand::thread_rng(),
                    rsa::Pkcs1v15Encrypt,
                    &message_bytes,
                )
                .unwrap();
        }

        self.socket
            .send(laminar::Packet::reliable_unordered(
                shared::server::DEFAULT_ADDRESS,
                message_bytes,
            ))
            .unwrap();
        self.socket.manual_poll(std::time::Instant::now());
    }

    pub fn connect(&mut self) {
        self.send(shared::ClientMessage::ConnectionRequest(
            self.private_key
                .to_public_key()
                .to_public_key_pem(rsa::pkcs1::LineEnding::CRLF)
                .unwrap(),
        ));

        self.state = ClientState::WaitingForServerPublicKey;
    }
    pub fn disconnect(&mut self) {
        self.send(shared::ClientMessage::DisconnectionRequest);

        debug!("Waiting for a disconnect response from the server");
        loop {
            if let shared::ServerMessage::Disconnected = self.wait_for_server_message() {
                break;
            }
        }
    }
    fn read_packet(&self, packet: &laminar::Packet) -> (bool, shared::ServerMessage) {
        if let Ok(message) = bincode::deserialize::<shared::ServerMessage>(packet.payload()) {
            return (false, message);
        }

        if let Ok(decyphered_bytes) = self
            .private_key
            .decrypt(rsa::Pkcs1v15Encrypt, packet.payload())
        {
            if let Ok(message) = bincode::deserialize::<shared::ServerMessage>(&decyphered_bytes) {
                return (true, message);
            }
        }
        panic!("Could not deserialize/decypher the message received from the server: {packet:?}");
    }

    pub fn wait_for_server_message(&mut self) -> shared::ServerMessage {
        loop {
            self.socket.manual_poll(std::time::Instant::now());
            while let Some(msg) = self.socket.recv() {
                if let laminar::SocketEvent::Packet(packet) = msg {
                    if packet.addr() == shared::server::DEFAULT_ADDRESS {
                        return self.read_packet(&packet).1;
                    } else {
                        println!("Unknown sender")
                    }
                }
            }
        }
    }

    fn handle_server_msg(&mut self, message: shared::ServerMessage) {
        if message == shared::ServerMessage::Disconnected {
            self.state = ClientState::Disconnected;
        }

        debug!("Reacting to {message:?}");
    }

    pub fn receive_server_messages(&mut self) {
        self.socket.manual_poll(std::time::Instant::now());
        while let Some(msg) = self.socket.recv() {
            match msg {
                laminar::SocketEvent::Packet(packet) => {
                    if packet.addr() != shared::server::DEFAULT_ADDRESS {
                        debug!("Unknow sender");
                        continue;
                    }

                    let (was_cyphered, message) = self.read_packet(&packet);

                    if !was_cyphered && !matches!(message, shared::ServerMessage::Connected(_)) {
                        error!("Client received a non-cyphered message that was no the server public key")
                    }

                    debug!("Server sent {:?}", message);
                    if message == shared::ServerMessage::Disconnected {
                        // self.disconnect()
                    }

                    match &self.state {
                        ClientState::Disconnected => {
                            debug!("Received a message from server while being disconnected");
                        }
                        ClientState::Connected(_server_public_key) => {
                            self.handle_server_msg(message)
                        }
                        ClientState::WaitingForServerPublicKey => {
                            if let shared::ServerMessage::Connected(server_public_key) = message {
                                // self.server_public_key = Some(
                                //     rsa::RsaPublicKey::from_public_key_pem(&server_public_key)
                                //         .unwrap(),
                                // );
                                self.state = ClientState::Connected(
                                    rsa::RsaPublicKey::from_public_key_pem(&server_public_key)
                                        .unwrap(),
                                );
                                debug!("Got the server's public key");
                                continue;
                            } else {
                                warn!("Client is waiting for the server's public key but the server sent another message: {message:?}")
                            }
                        }
                    }
                }
                laminar::SocketEvent::Connect(_) => {
                    // debug!("Connect")
                }
                laminar::SocketEvent::Timeout(_) => {
                    // debug!("Timeout")
                }
                laminar::SocketEvent::Disconnect(_) => {
                    // debug!("Disconnect")
                }
            }
        }
    }
}
