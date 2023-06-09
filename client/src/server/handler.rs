pub struct ServerHandle {
    channel: shared::threading::Channel<crate::server::Message>,
    pub account_state: Option<crate::server::AccountState>,
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
            account_state: None,
        }
    }

    pub fn update(&mut self) {
        if let Ok(msg) = self.channel.try_recv() {
            match msg {
                crate::server::Message::LoginResponse(response) => match response {
                    Ok(id) => {
                        self.account_state = Some(crate::server::AccountState { id, fs: None })
                    }
                    Err(e) => error!("Error while trying to log in: {e}"),
                },
                crate::server::Message::LogoutResponse(response) => match response {
                    Ok(_) => self.account_state = None,
                    Err(e) => error!("Error while trying to log out: {e}"),
                },
                crate::server::Message::FileScanUpdate(scan) => {
                    if let Some(acc_state) = &mut self.account_state {
                        acc_state.fs = Some(scan)
                    } else {
                        warn!("Received a file scan but im not logged in")
                    }
                }
                crate::server::Message::ChangeDirectory(new_dir) => {}

                crate::server::Message::LoginRequest { .. }
                | crate::server::Message::LogoutRequest { .. } => {
                    unreachable!()
                }
            };

            // self.channel.send(response).unwrap()
        }
    }

    pub fn login(&mut self, username: &str, password: &str) {
        self.channel
            .send(crate::server::Message::LoginRequest {
                username: username.to_string(),
                password: password.to_string(),
            })
            .unwrap();
    }

    pub fn logout(&mut self) {
        if let Some(acc_state) = &self.account_state {
            self.channel
                .send(crate::server::Message::LogoutRequest { id: acc_state.id })
                .unwrap();
        } else {
            error!("Tried to log out but user is not logged in")
        }
    }

    pub fn cd(&mut self, new_dir: String) {
        self.channel
            .send(crate::server::Message::ChangeDirectory(new_dir))
            .unwrap();
    }
}
