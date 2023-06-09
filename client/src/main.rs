#[macro_use]
extern crate log;

mod server;
mod ui;

fn main() {
    shared::logger::init(log::LevelFilter::Debug, None);

    ui::run()
}
