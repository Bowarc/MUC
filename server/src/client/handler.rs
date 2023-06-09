pub struct ClientHandle {
    channel: shared::threading::Channel<crate::client::Message>,
    pub ip: std::net::SocketAddr,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl ClientHandle {
    pub fn new(stream: std::net::TcpStream, ip: std::net::SocketAddr) -> Self {
        let (channel1, channel2) = shared::threading::Channel::new_pair();

        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

        let running_thread = running.clone();
        std::thread::spawn(move || {
            let mut client = crate::client::Client::new(stream, channel1, ip, running_thread);

            client.run()
        });

        Self {
            channel: channel2,
            ip,
            running,
        }
    }

    pub fn update(
        &mut self,
        account_mgr: &mut crate::account_manager::AccountManager,
    ) -> Result<(), shared::error::ChannelError<crate::client::Message>> {
        if let Ok(msg) = self.channel.try_recv() {
            let response = match msg {
                crate::client::Message::LoginRequest { username, password } => {
                    crate::client::Message::LoginResponse(
                        account_mgr.login(username, password, self.ip),
                    )
                }
                crate::client::Message::LogoutRequest { id } => {
                    crate::client::Message::LogoutResponse(account_mgr.logout(id))
                }
                crate::client::Message::LoginResponse { .. }
                | crate::client::Message::LogoutResponse { .. } => {
                    unreachable!()
                }
            };

            self.channel.send(response)?
        }

        if !self.running.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(shared::error::ChannelError::Other(
                "Target thread has exited",
            ));
        }
        Ok(())
    }
}
