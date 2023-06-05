fn main() {
    shared::logger::init(None);

    let stream = std::net::TcpStream::connect(shared::networking::DEFAULT_ADDRESS).unwrap();
    let mut socket = shared::networking::Socket::<
        shared::networking::ServerMessage,
        shared::networking::ClientMessage,
    >::new(stream);

    socket
        .send(shared::networking::ClientMessage::LoginRequest {
            username: "username".into(),
            password: "pw".into(),
        })
        .unwrap();

    let msg = socket.recv();

    dbg!(&msg);

    let msg = msg.unwrap();
    let id = if let shared::networking::ServerMessage::LoginResponse(resp) = msg {
        resp.unwrap()
    } else {
        panic!("Could not get account id out of message")
    };

    std::thread::sleep(std::time::Duration::from_secs(5));

    socket
        .send(shared::networking::ClientMessage::LogoutRequest {
            id: uuid::Uuid::parse_str(&id).unwrap(),
        })
        .unwrap();

    let msg = socket.recv();
    dbg!(msg);
}
