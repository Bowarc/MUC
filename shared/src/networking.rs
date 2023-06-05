use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const HEADER_SIZE: usize = std::mem::size_of::<PacketHeader>();

pub const DEFAULT_ADDRESS: std::net::SocketAddr = std::net::SocketAddr::V4(
    std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(127, 0, 0, 1), 44415),
);

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
// ofc don't use type that can change size (such as Vec) so the size of the struct stays the same as the constant
pub struct PacketHeader {
    size: usize,
}

// I don't like how streams work so i'll make a simple socket-like, packet-based struct wrapper
pub struct Socket<R, W> {
    stream: std::net::TcpStream,
    read_type: std::marker::PhantomData<R>,
    write_type: std::marker::PhantomData<W>,
    last_header: Option<PacketHeader>,
}

#[derive(thiserror::Error, Debug)]
pub enum SocketError {
    #[error("This should not be used outside tests")]
    TestError,
    #[error("Error when serializing or deserializing: {0}")]
    DeSerialization(#[from] bincode::Error),
    #[error("std::io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Still waiting for more data")]
    NotEnoughData(usize, usize),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Text(String),
    LoginRequest { username: String, password: String },
    LogoutRequest { id: uuid::Uuid },
}
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Text(String),
    LoginResponse(Result<String, String>),
    LogoutResponse(Result<(), String>),
}

impl<R: DeserializeOwned + std::fmt::Debug, W: Serialize + std::fmt::Debug> Socket<R, W> {
    pub fn new(stream: std::net::TcpStream) -> Self {
        Self {
            stream,
            read_type: std::marker::PhantomData,
            write_type: std::marker::PhantomData,
            last_header: None,
        }
    }
    pub fn send(&mut self, message: W) -> Result<(), SocketError> {
        use std::io::Write as _;

        // debug!("send {message:?}");

        let message_bytes = bincode::serialize(&message)?;
        trace!("Serializing message.. Done, {} bytes", message_bytes.len());

        let header = PacketHeader::new(message_bytes.len());
        trace!("Creating header.. Done, {header:?}");

        let header_bytes = bincode::serialize(&header)?;
        trace!("Serializing header.. Done, {} bytes", header_bytes.len());

        // idk if panicking is a good idea
        // assert_eq!(header_bytes.len(), HEADER_SIZE);
        if header_bytes.len() != HEADER_SIZE {
            return Err(SocketError::DeSerialization(Box::new(bincode::ErrorKind::Custom(format!("The length of the serialized header is not equal to the HEADER_SIZE constant ({HEADER_SIZE})"))),));
        }

        self.stream.write_all(&header_bytes)?;
        trace!("Writing header to stream.. Ok ({:?})", &header_bytes);

        self.stream.write_all(&message_bytes)?;
        trace!("Writing message to stream.. Ok({:?})", &message_bytes);

        Ok(())
    }
    pub fn recv(&mut self) -> Result<R, SocketError> {
        // debug!("recv");

        // well, this method doesn't fix the problem
        let header = match self.last_header {
            Some(header) => {
                debug!("Using saved header: {header:?}");
                header
            }
            None => {
                let header = self.try_get::<PacketHeader>(HEADER_SIZE)?;

                self.last_header = Some(header);
                header
            }
        };

        // let mut header_buffer: [u8; HEADER_SIZE] = [0; HEADER_SIZE];

        // self.stream.read_exact(&mut header_buffer)?;
        // print!("Reading header.. Done, {} bytes", header_buffer.len());

        // let header: PacketHeader = bincode::deserialize(&header_buffer)?;
        // print!("Deserializing header.. Done: {header:?}");

        // self.last_header = Some(header);

        let message = self.try_get::<R>(header.size)?;

        self.last_header = None;

        Ok(message)
    }

    fn try_get<T: serde::de::DeserializeOwned + std::fmt::Debug>(
        &mut self,
        target_size: usize,
    ) -> Result<T, SocketError> {
        use std::io::Read as _;
        let mut peek_buffer = vec![0; target_size];

        let read_len = self.stream.peek(&mut peek_buffer)?;

        if read_len != 0 {
            debug!(
                "Peeking steam, looking for {} bytes.. Done, found {} bytes",
                target_size, read_len
            );
        }

        if read_len != target_size {
            if read_len != 0 {
                warn!("Read {} but was waiting for {}", read_len, target_size);
            }
            return Err(SocketError::NotEnoughData(target_size, read_len));
        }

        let mut message_buffer = vec![0; target_size];

        self.stream.read_exact(&mut message_buffer)?;

        let message: T = bincode::deserialize(&message_buffer)?;
        debug!("Deserializing message.. Done, {message:?}");

        Ok(message)
    }

    pub fn local_addr(&self) -> std::net::SocketAddr {
        self.stream.local_addr().unwrap()
    }

    pub fn remote_addr(&self) -> std::net::SocketAddr {
        self.stream.peer_addr().unwrap()
    }
}

impl PacketHeader {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}
