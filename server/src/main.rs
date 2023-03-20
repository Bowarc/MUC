#[macro_use]
extern crate log;
mod client;
mod server;

use shared::{logger, server::*};

fn main() {
    // let msg = shared::Message::None;

    let logger_config = logger::LoggerConfig::new().set_level(log::LevelFilter::Trace);

    logger::init(logger_config);
    // logger::test();
    debug!("Hello world! ");
    let mut server = server::Server::new(laminar::Socket::bind(DEFAULT_ADDRESS).unwrap());

    loop {
        // std::thread::sleep(std::time::Duration::from_millis(50));
        // debug!("Server loop");
        server.handle_client_messages();
    }
}
