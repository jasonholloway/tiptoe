mod common;
mod msg;
mod peer;
mod roost;
mod visits;
mod server;

use server::Server;
use std::{net::{TcpListener, TcpStream}, io::Write};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:17878").unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut log = TcpStream::connect("vm:17879").unwrap();
    writeln!(log, "\t\t\tStart").unwrap();

    let mut server = Server::new();
    server.pump(listener, &mut log);
}

