use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const HEADER_SIZE: usize = std::mem::size_of::<PacketHeader>();

pub const DEFAULT_ADDRESS: std::net::SocketAddr = std::net::SocketAddr::V4(
    std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(127, 0, 0, 1), 14045),
);

#[derive(Serialize, Deserialize, Debug)]
// ofc don't use type that can change size (such as Vec) so the size of the struct stays the same as the constant
pub struct PacketHeader {
    size: usize,
}

// I don't like how streams work so i'll make a simple socket-like, packet-based struct wrapper
pub struct Socket<R, W> {
    stream: std::net::TcpStream,
    read_type: std::marker::PhantomData<R>,
    write_type: std::marker::PhantomData<W>,
}

#[derive(thiserror::Error, Debug)]
pub enum SocketError {
    #[error("This should not be used outside tests")]
    TestError,
    #[error("Error when serializing or deserializing: {0}")]
    DeSerialization(#[from] bincode::Error),
    #[error("std::io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Text(String),
}
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Text(String),
}

impl<R: DeserializeOwned + std::fmt::Debug, W: Serialize + std::fmt::Debug> Socket<R, W> {
    pub fn new(stream: std::net::TcpStream) -> Self {
        Self {
            stream,
            read_type: std::marker::PhantomData,
            write_type: std::marker::PhantomData,
        }
    }
    pub fn send(&mut self, message: W) -> Result<(), SocketError> {
        use std::io::Write as _;

        // trace!("Serializing message..");
        let message_bytes = bincode::serialize(&message)?;
        // trace!("Serializing message.. Done, {} bytes", message_bytes.len());

        // trace!("Creating header..");
        let header = PacketHeader::new(message_bytes.len());
        // trace!("Creating header.. Done, {header:?}");

        // trace!("Serializing header..");
        let header_bytes = bincode::serialize(&header)?;
        // trace!("Serializing header.. Done, {} bytes", header_bytes.len());

        // idk if panicking is a good idea
        assert_eq!(header_bytes.len(), HEADER_SIZE);
        // if header_bytes.len() != HEADER_SIZE {
        //     return Err(SocketError::DeSerialization(Box::new(bincode::ErrorKind::Custom("The length of the serialized header is not equal to the HEADER_SIZE constant ({HEADER_SIZE})".into())),));
        // }

        // trace!("Writing header to stream..");
        self.stream.write_all(&header_bytes)?;
        // trace!("Writing header to stream.. Ok");
        // trace!("Writing message to stream..");
        self.stream.write_all(&message_bytes)?;
        // trace!("Writing message to stream.. Ok");

        // trace!("Exiting send function");
        Ok(())
    }
    pub fn recv(&mut self) -> Result<R, SocketError> {
        use std::io::Read as _;

        let mut header_buffer: [u8; HEADER_SIZE] = [0; HEADER_SIZE];

        // trace!("Reading header..");
        self.stream.read_exact(&mut header_buffer)?;
        // trace!("Reading header.. Done, {} bytes", header_buffer.len());

        // trace!("Deserializing header..");
        let header: PacketHeader = bincode::deserialize(&header_buffer)?;
        // trace!("Deserializing header.. Done: {header:?}");

        let mut message_buffer = vec![0; header.size];

        // trace!("Reading message ({} bytes)..", header.size);
        self.stream.read_exact(&mut message_buffer)?;
        // trace!(
        //     "Reading message ({} bytes).. Done, {} bytes",
        //     header.size,
        //     message_buffer.len()
        // );

        // trace!("Deserializing message..");
        let message = bincode::deserialize(&message_buffer)?;
        // trace!("Deserializing message.. Done, {message:?}");

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
