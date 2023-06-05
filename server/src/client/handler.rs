pub struct ClientHandle {
    channel: crate::threading::Channel<Message>,
    ip: std::net::SocketAddr,
}

impl ClientHandle {
    pub fn new(stream: std::net::TcpStream, ip: std::net::SocketAddr) -> Self {
        let (channel1, channel2) = crate::threading::Channel::new_pair();

        std::thread::spawn(move || {
            let mut client = Client::new(stream, channel1);

            client.run()
        });

        Self {
            channel: channel2,
            ip,
        }
    }

    pub fn update(&mut self, account_mgr: &mut crate::account_manager::AccountManager) {
        if let Ok(msg) = self.channel.try_recv() {
            let response = match msg {
                Message::LoginRequest { username, password } => {
                    Message::LoginResponse(account_mgr.login(username, password, self.ip))
                }
                Message::LogoutRequest { id } => Message::LogoutResponse(account_mgr.logout(id)),
                Message::LoginResponse { .. } | Message::LogoutResponse { .. } => {
                    unreachable!()
                }
            };

            if let Err(e) = self.channel.send(response) {
                error!("{e}")
            }
        }
    }
}
