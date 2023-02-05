mod common;
mod peer;
mod roost;
mod server;
mod lossy_stack;

use common::{Cmd, Talk, ReadResult};
use peer::Peer;
use server::{Server, State};
use std::{net::{TcpListener, TcpStream}, io::{Write, ErrorKind, BufReader, BufRead}, time::{Duration, Instant}, thread::sleep, str::from_utf8};

fn main() {
    let delay = Duration::from_millis(50);
    
    let listener = TcpListener::bind("127.0.0.1:17878").unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut log = TcpStream::connect("127.0.0.1:17879").unwrap();
    writeln!(log, "\t\t\tStart").unwrap();

    let mut server: Server<TcpTalker> = Server::new(Instant::now());
    let mut state = State::Starting;

    loop {
        let now = Instant::now();

        let mut work_done = match listener.accept() {
            Ok((stream, address)) => {
                stream.set_nonblocking(true).unwrap();
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




struct TcpTalker {
    input: BufReader<TcpStream>,
    output: TcpStream,
    buffer: Vec<u8>,
    active: bool
}

impl TcpTalker {
    pub fn new(stream: TcpStream) -> TcpTalker {
        TcpTalker {
            input: BufReader::new(stream.try_clone().unwrap()),
            output: stream,
            buffer: Vec::new(),
            active: true
        }
    }
}

impl Talk for TcpTalker {
    fn read(&mut self) -> common::ReadResult<String> {
        if !self.active {
            ReadResult::Stop
        }
        else {
            match self.input.read_until(b'\n', &mut self.buffer) {
                Ok(0) => {
                    self.active = false;
                    ReadResult::Stop
                }
                Ok(_) => {
                    let result = self.buffer
                        .split_last()
                        .and_then(|(_,l)| from_utf8(l).ok())
                        .map(|s| ReadResult::Yield(s.trim().to_string()))
                        .unwrap_or(ReadResult::Continue);

                    self.buffer.clear();

                    result
                }, 
                Err(e) if e.kind() == ErrorKind::WouldBlock => ReadResult::Continue,
                Err(e) => {
                    println!("Unexpected read error {e:?}");
                    self.active = false;
                    ReadResult::Stop
                }
            }
        }
    }
}

impl std::fmt::Write for TcpTalker {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        writeln!(self.output, "{}", s).expect("Failed to write to tcp");
        Ok(())
    }
}

