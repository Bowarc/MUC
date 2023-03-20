#[macro_use]
extern crate log;
use shared::{client::*, logger};

mod client;

fn main() {
    let logger_config = logger::LoggerConfig::new().set_level(log::LevelFilter::Trace);

    logger::init(logger_config);
    // logger::test();
    debug!("Hello world! {:?}", shared::server::DEFAULT_ADDRESS);

    let mut client = client::Client::new(shared::server::DEFAULT_ADDRESS);
    client.connect();
    debug!("Connected");

    let stdin = std::io::stdin();

    let mut s_buffer = String::new();
    loop {
        // ask the user for a message
        // s_buffer.clear();
        // stdin.read_line(&mut s_buffer).unwrap();
        // let line = s_buffer.replace(|x| x == '\n' || x == '\r', "");

        // // transform the user's message into a shared::Message
        // let message = shared::ClientMessage::Text(line);

        // // send the message to the server
        // client.send(message);

        // custom lag
        // std::thread::sleep(std::time::Duration::from_millis(500));
        // debug!("Listening the server for any message");
        // handle the messages of the server
        client.receive_server_messages()
    }
}
