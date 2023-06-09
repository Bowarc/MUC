#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate derivative;

mod account_manager;
mod client;
mod error;
mod file;

struct Server {
    clients: Vec<client::ClientHandle>,
    listener: std::net::TcpListener,
    account_manager: account_manager::AccountManager,
}

impl Server {
    fn new() -> Self {
        let listener = std::net::TcpListener::bind(shared::networking::DEFAULT_ADDRESS).unwrap();
        listener.set_nonblocking(true).unwrap();

        Self {
            clients: vec![],
            listener,
            account_manager: account_manager::AccountManager::load(),
        }
    }
    fn update(&mut self) {
        // debug!(
        //     "Connected accounts: {:?}",
        //     self.account_manager.connected_accounts
        // );

        self.clients.retain_mut(|handle| {
            debug!("updating ({})", handle.ip);
            if let Err(e) = handle.update(&mut self.account_manager) {
                error!(
                    "An error occured while updating client handle ({}) {e}, closing the handle",
                    handle.ip
                );
                false
            } else {
                true
            }
        });

        match self.listener.accept() {
            Ok((stream, addr)) => {
                debug!("New client {addr:?}");
                // stream.set_nodelay(true).unwrap(); // ?

                self.clients.push(client::ClientHandle::new(stream, addr));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // wait until network socket is ready, typically implemented
                // via platform-specific APIs such as epoll or IOCP
                // println!("Would block");
                // continue;

                // About this part, as the implementation is non-blocking,
                // i'll assume that the program will do some other job before getting back to this part,
                // therefore the socket will have time to do it's things
            }

            Err(e) => {
                error!("Error while listening for clients: {e:?}");
            }
        }
    }

    fn update_client_handles(&mut self) {}
}

const TARGET_TPS: f32 = 10.;

fn main() {
    shared::logger::init(log::LevelFilter::Trace, None);

    // Dummy first account

    // let acc = account_manager::Account::new("Bowarc", "Password1");

    // let mut acc_mgr = account_manager::AccountManager::new_empty();

    // acc_mgr.register(acc);
    // acc_mgr.save()

    //
    let running = set_up_ctrlc();

    let mut server = Server::new();

    debug!("{:?}", server.account_manager);

    let mut loop_helper = spin_sleep::LoopHelper::builder()
        .report_interval_s(0.5)
        .build_with_target_rate(TARGET_TPS);

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        loop_helper.loop_start();
        server.update();

        loop_helper.loop_sleep();
    }

    server.account_manager.exit_cleanup();

    debug!("Saving accounts and quitting");
}

fn set_up_ctrlc() -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    running
}
