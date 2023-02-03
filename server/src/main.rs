mod common;
mod msg;
mod peer;
mod roost;
mod visits;
mod server;

use server::{Server, State};
use std::{net::{TcpListener, TcpStream}, io::Write, time::{Duration, Instant}, thread::sleep};

fn main() {
    let delay = Duration::from_millis(50);
    
    let mut listener = TcpListener::bind("127.0.0.1:17878").unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut log = TcpStream::connect("127.0.0.1:17879").unwrap();
    writeln!(log, "\t\t\tStart").unwrap();

    let mut server = Server::new(Instant::now());
    let mut state = State::AtRest;

    loop {
        let result = server.pump(state, &mut listener, Instant::now(), &mut log);
        state = result.0;
        let work_done = result.1;

        if !work_done {
            sleep(delay);
        }
    }
}

