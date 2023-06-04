fn main() {
    let stream = std::net::TcpStream::connect(shared::networking::DEFAULT_ADDRESS).unwrap();
    let mut socket = shared::networking::Socket::<
        shared::networking::ServerMessage,
        shared::networking::ClientMessage,
    >::new(stream);

    socket
        .send(shared::networking::ClientMessage::Text(String::from(
            "Hellow",
        )))
        .unwrap();

    let msg = socket.recv();
    dbg!(msg);
}
