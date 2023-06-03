#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;

mod account_manager;
mod client_handler;
mod file;

struct Server {
    clients: Vec<client_handler::Client>,
    listener: std::net::TcpListener,
}

impl Server {
    fn new() -> Self {
        let listener = std::net::TcpListener::bind(shared::networking::DEFAULT_ADDRESS).unwrap();
        listener.set_nonblocking(true).unwrap();

        Self {
            clients: vec![],
            listener,
        }
    }
    fn update(&mut self) {
        debug!("Update")
    }
}

const TARGET_TPS: f32 = 10.;

fn main() {
    shared::logger::init(None);

    let mut server = Server::new();

    let mut loop_helper = spin_sleep::LoopHelper::builder()
        .report_interval_s(0.5)
        .build_with_target_rate(TARGET_TPS);

    loop {
        loop_helper.loop_start();
        server.update();

        loop_helper.loop_sleep();
    }
}
