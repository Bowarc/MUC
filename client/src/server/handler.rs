pub struct ServerHandle {
    channel: shared::threading::Channel<crate::server::Message>,
    pub ip: Option<std::net::SocketAddr>,
}

impl ServerHandle {
    pub fn new() -> Self {
        let (channel1, channel2) = shared::threading::Channel::new_pair();

        std::thread::spawn(move || {
            let mut server = crate::server::Server::new(channel1);

            server.run()
        });

        Self {
            channel: channel2,
            ip: None,
        }
    }

    pub fn update(&mut self) {}
}
