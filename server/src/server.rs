use crate::client::Client;
use rsa::pkcs8::DecodePublicKey;
use rsa::pkcs8::EncodePublicKey;

pub struct Server {
    socket: laminar::Socket,
    private_key: rsa::RsaPrivateKey,
    clients: Vec<Client>,
}

impl Server {
    pub fn new(socket: laminar::Socket) -> Self {
        Self {
            socket,
            private_key: rsa::RsaPrivateKey::new(&mut rand::thread_rng(), 2048).unwrap(),
            clients: Vec::new(),
        }
    }

    pub fn send(&mut self, client_id: shared::id::Id, message: shared::ServerMessage) {
        let client = self.find_client_by_id(client_id).unwrap();

        debug!("Sent {message:?} to {client}");

        // let message_bytes = if !matches!(message, shared::ServerMessage::Connected(..)) {
        //     client.encrypt(message)
        // } else {
        //     bincode::serialize(&message).unwrap()
        // };

        let message_bytes = if let shared::ServerMessage::Connected(_) = message {
            // Do not encrypt the message that is sending the public key
            bincode::serialize(&message).unwrap()
        } else {
            client.encrypt(message)
        };

        self.socket
            .send(laminar::Packet::reliable_unordered(
                client.addr,
                message_bytes,
            ))
            .unwrap();
        self.socket.manual_poll(std::time::Instant::now());
    }

    fn read_packet(&self, packet: &laminar::Packet) -> (bool, shared::ClientMessage) {
        if let Ok(message) = bincode::deserialize::<shared::ClientMessage>(packet.payload()) {
            return (false, message);
        }

        if let Ok(decyphered_bytes) = self
            .private_key
            .decrypt(rsa::Pkcs1v15Encrypt, packet.payload())
        {
            if let Ok(message) = bincode::deserialize::<shared::ClientMessage>(&decyphered_bytes) {
                return (true, message);
            }
        }
        panic!("Could not deserialize/decypher the message received from the server: {packet:?}");
    }

    fn handle_packet(&mut self, packet: laminar::Packet) {
        let (was_cyphered, message) = self.read_packet(&packet);

        if !was_cyphered && !matches!(message, shared::ClientMessage::ConnectionRequest(_)) {
            error!("Server received a non-cyphered message that was not the client's connection request")
        }

        debug!("Received {:?} from {:?}", message, packet.addr());
        match message {
            shared::ClientMessage::ConnectionRequest(client_public_key) => {
                if let Some(client) = self.find_client_by_addr(packet.addr()) {
                    warn!("Server received a ConnectionRequest from a addr already registered as a client");
                    self.send(client.id, shared::ServerMessage::AlreadyConnected);
                    return;
                }

                self.register_client(
                    packet.addr(),
                    rsa::RsaPublicKey::from_public_key_pem(&client_public_key).unwrap(),
                );
            }
            shared::ClientMessage::DisconnectionRequest => {
                if let Some(client) = self.find_client_by_addr(packet.addr()) {
                    self.remove_client(client.id)
                } else {
                    warn!(
                        "Server tried to remove a client that is not registered: {}",
                        packet.addr()
                    )
                }
            }
            shared::ClientMessage::Text(txt) => {
                if self.find_client_by_addr(packet.addr()).is_none() {
                    warn!("Received a message from something that is not a client");
                } else {
                    debug!("Server received message - {txt}");
                }
            }
            shared::ClientMessage::None => {
                warn!("Server received None")
            }
        }
    }

    pub fn register_client(
        &mut self,
        addr: std::net::SocketAddr,
        client_public_key: rsa::RsaPublicKey,
    ) {
        if let Some(_client) = self.find_client_by_addr(addr) {
            // This addr has already been registered
            warn!("Tried to registed an address that already had a client: {addr:?}");
            return;
        }
        let new_client = Client::new(addr, client_public_key);
        debug!("New client has connected: {}", new_client);
        let new_client_id = new_client.id;
        self.clients.push(new_client);
        self.send(
            new_client_id,
            shared::ServerMessage::Connected(
                self.private_key
                    .to_public_key()
                    .to_public_key_pem(rsa::pkcs1::LineEnding::CRLF)
                    .unwrap(),
            ),
        );
    }
    pub fn remove_client(&mut self, client_id: shared::id::Id) {
        self.clients.retain(|c| c.id != client_id);
    }

    pub fn handle_client_messages(&mut self) {
        self.socket.manual_poll(std::time::Instant::now());
        while let Ok(event) = self.socket.get_event_receiver().try_recv() {
            match event {
                laminar::SocketEvent::Packet(packet) => self.handle_packet(packet),
                laminar::SocketEvent::Connect(_address) => {
                    // self.register_client(address);
                }
                laminar::SocketEvent::Timeout(address) => {
                    debug!("{} timed out", address);
                    if let Some(client) = self.find_client_by_addr(address) {
                        self.send(client.id, shared::ServerMessage::Disconnected);
                        self.remove_client(self.find_client_by_addr(address).unwrap().id);
                    }
                }
                laminar::SocketEvent::Disconnect(address) => {
                    debug!("{} disconnected", address);
                    if let Some(client) = self.find_client_by_addr(address) {
                        self.send(client.id, shared::ServerMessage::Disconnected);
                        self.remove_client(self.find_client_by_addr(address).unwrap().id);
                    }
                }
            }
        }
    }

    fn find_client_by_addr(&self, addr: std::net::SocketAddr) -> Option<&Client> {
        self.clients.iter().find(|c| c.addr == addr)
    }

    fn find_client_by_id(&self, id: shared::id::Id) -> Option<&Client> {
        self.clients.iter().find(|c| c.id == id)
    }
}
