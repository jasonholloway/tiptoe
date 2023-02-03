mod common;
mod msg;
mod peer;
mod roost;
mod visits;
mod server;
mod lossy_stack;

use server::{Server, State};
use std::{net::{TcpListener, TcpStream}, io::Write, time::{Duration, Instant}, thread::sleep};

fn main() {
    let delay = Duration::from_millis(50);
    
    let mut listener = TcpListener::bind("127.0.0.1:17878").unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut log = TcpStream::connect("127.0.0.1:17879").unwrap();
    writeln!(log, "\t\t\tStart").unwrap();

    let mut server = Server::new(Instant::now());
    let mut state = State::Start;

    loop {
        let now = Instant::now();

        let (new_state, work_done) = server.pump(state, &mut listener, now, &mut log);
        state = new_state;

        if !work_done {
            sleep(delay);
        }
    }
}

