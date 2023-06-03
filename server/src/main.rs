#[macro_use]
extern crate log;

fn main() {
    shared::logger::init(None);

    debug!("Hellow");
}
