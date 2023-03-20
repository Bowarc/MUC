use rsa::PublicKey;

#[derive(Clone)]
pub struct Client {
    pub addr: std::net::SocketAddr,
    pub id: shared::id::Id,
    pub public_key: rsa::RsaPublicKey,
    // pub conn: ClientConnection,
}

#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub token: String,
}

impl Client {
    pub fn new(addr: std::net::SocketAddr, public_key: rsa::RsaPublicKey) -> Self {
        Self {
            addr,
            id: shared::id::Id::new(),
            public_key,
        }
    }
    pub fn encrypt(&self, message: shared::ServerMessage) -> Vec<u8> {
        self.public_key
            .encrypt(
                &mut rand::thread_rng(),
                rsa::Pkcs1v15Encrypt,
                &bincode::serialize(&message).unwrap(),
            )
            .unwrap()
    }
}

impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client: {{ addr: {}, id: {}}}", self.addr, self.id)
    }
}
