#[macro_use]
extern crate log;

fn main() {
    shared::logger::init(log::LevelFilter::Debug, None);

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

    debug!("Waiting for sever response");

    let msg = loop {
        match socket.recv() {
            Ok(msg) => break msg,
            Err(e) => {
                // warn!("{e:?}");
            }
        }
    };

    dbg!(&msg);

    let id = if let shared::networking::ServerMessage::LoginResponse(resp) = msg {
        resp.unwrap()
    } else {
        panic!("Could not get account id out of message")
    };

    std::thread::sleep(std::time::Duration::from_secs(5));

    socket
        .send(shared::networking::ClientMessage::LogoutRequest { id })
        .unwrap();

    loop {
        if let Ok(msg) = socket.recv() {
            dbg!(msg);
            break;
        }
    }
}
