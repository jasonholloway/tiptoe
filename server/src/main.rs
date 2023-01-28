mod common;
mod msg;
mod peer;
mod roost;
mod visits;
mod traits;
mod server;

use server::Server;
use std::io::BufReader;
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:17878").unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut server = Server::new();
    server.pump(listener);
}

