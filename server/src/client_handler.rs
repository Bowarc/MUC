struct Client {
    socket: shared::networking::Socket<
        shared::networking::ClientMessage,
        shared::networking::ServerMessage,
    >,
}

pub enum Message {
    ConnectionRequest { username: String, password: String },
    ConnectionConfirmation { id: uuid::Uuid },
}

pub struct ClientHandle {
    channel: crate::threading::Channel<Message>,
}

impl Client {
    fn new(stream: std::net::TcpStream) -> Self {
        Self {
            socket: shared::networking::Socket::<
                shared::networking::ClientMessage,
                shared::networking::ServerMessage,
            >::new(stream),
        }
    }
}

impl ClientHandle {
    // pub fn new(stream: std::net::TcpStream) -> Self {
    //     let client = Client {
    //         socket: shared::networking::Socket::<
    //             shared::networking::ClientMessage,
    //             shared::networking::ServerMessage,
    //         >::new(stream),
    //     };

    //     let channel = crate::threading::Channel::new_pair();

    //     Self {}
    // }
}
