#![feature(assert_matches)]

mod common;
mod peer;
mod roost;
mod server;
mod tcp_talker;

use crate::server::Server;
use common::Cmd;
use peer::Peer;
use server::State;
use tcp_talker::TcpTalker;
use std::{net::{TcpListener, TcpStream}, io::{Write, ErrorKind}, time::{Duration, Instant}, thread::sleep};

fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:17878")?;
    listener.set_nonblocking(true)?;

    match TcpStream::connect("127.0.0.1:17879") {
        Ok(stream) => run(listener, stream),
        Err(_) => run(listener, std::io::stderr())
    }
}

fn run<L: Write>(listener: TcpListener, mut log: L) -> Result<(), std::io::Error> {
    let delay = Duration::from_millis(50);

    writeln!(log, "\t\t\tStart")?;

    let mut server: Server<TcpTalker> = Server::new(Instant::now());
    let mut state = State::Starting;

    loop {
        let now = Instant::now();

        let mut work_done = match listener.accept() {
            Ok((stream, address)) => {
                stream.set_nonblocking(true)?;
                server.enqueue(Cmd::Connect(Peer::new(&address.to_string(), TcpTalker::new(stream))));
                true
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => false,
            Err(e) => {
                panic!("Unexpected connect error {e:?}")
            }
        };

        let (s2, w2) = server.pump(state, now, &mut log);
        state = s2;
        work_done |= w2;

        if !work_done {
            sleep(delay);
        }
    }
}
