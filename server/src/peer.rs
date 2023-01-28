use crate::{BufReader, msg::{Msg, self}, common::RR, traits::Actions};
use core::fmt::Debug;
use std::{net::{TcpStream, SocketAddr}, io::{BufRead, ErrorKind}, str::from_utf8};

pub struct Peer {
    pub input: PeerInput,
    pub output: PeerOutput,
    pub state: PeerState,
}

pub struct PeerInput {
    reader: BufReader<TcpStream>,
    buffer: Vec<u8>,
}

pub struct PeerOutput {
    writer: TcpStream,
		buffer: Vec<u8>
}

pub struct PeerState {
    mode: PeerMode,
    line_mode: LineMode,
    pub tag: String,
    addr: SocketAddr
}

impl Peer {
    pub fn new(addr: SocketAddr, stream: TcpStream) -> Peer {
        let stream2 = stream.try_clone().unwrap();
        Peer {
            input: PeerInput {
                reader: BufReader::new(stream),
                buffer: Vec::new()
            },
            output: PeerOutput {
                writer: stream2,
                buffer: Vec::new()
            },
            state: PeerState {
                mode: PeerMode::Start,
                line_mode: LineMode::Basic,
                tag: String::new(),
                addr
            },
        }
    }

    pub fn read(&mut self) -> Option<Msg> {
        let s = &mut self.state;
        let input = &mut self.input;

        let msg = match (&s.mode, &s.line_mode) {
            (PeerMode::Closed, _) => None,

            (_, LineMode::Basic) => {
                match input.reader.read_until(b'\n', &mut input.buffer) {
                    Ok(0) => {
                        self.close();
                        None
                    }
                    Ok(_) => {
                        let msg = input.buffer
                            .split_last()
                            .and_then(|(_,l)| from_utf8(l).ok())
                            .and_then(msg::try_parse);
                        
                        input.buffer.clear();

                        msg
                    }, 
                    Err(e) if e.kind() == ErrorKind::WouldBlock => None,
                    Err(e) => {
                        println!("Unexpected read error {e:?}");
                        self.close();
                        None
                    }
                }
            }

            (_, LineMode::Browser) => {
                match input.reader.read_until(b';', &mut input.buffer) {
                    Ok(0) => {
                        self.close();
                        None
                    },
                    Ok(_) => {
                        let msg = input.buffer
                            .split_last()
                            .map(|(_,l)| from_utf8(l).unwrap())
                            .and_then(|s| s.split_once('*'))
                            .and_then(|(_,l)| msg::try_parse(l));

                        input.buffer.clear();

                        msg
                    }, 
                    Err(e) if e.kind() == ErrorKind::WouldBlock => None,
                    Err(e) => {
                        println!("Unexpected read error {e:?}");
                        self.close();
                        None
                    }
                }
            }
        };

        for m in msg.iter() {
            self.log(m);
        }

        msg
    }

    pub fn write(&mut self, msg: Msg) -> Result<(), std::io::Error> {
				match self.state.line_mode {
						_ => {
								println!("O {:?}", &self.output.buffer);
								msg::write(msg, &mut self.output.writer)
						}
				}
    }

    fn close(&mut self) -> () {
        self.state.mode = PeerMode::Closed;
        self.log("Closed");
    }

    fn log<O: Debug>(&self, o: O) {
        println!("{:?}: {:?}", self.state.addr, o)
    }
}

impl PeerState {
    pub fn handle<A: Actions>(&mut self, a: &mut A, rc: &RR<Peer>, msg: Msg) -> () {
        match (&self.mode, msg) {
            (PeerMode::Start, Msg::Hello(new_tag, new_parse_mode)) => {
                a.perch(&new_tag, rc.clone());

                self.tag = new_tag;
                self.line_mode = new_parse_mode;
                self.mode = PeerMode::Active;
            }
            (PeerMode::Active, Msg::Visited(r)) => {
                a.push_visit(self.tag.to_string(), r.to_string());
            }

            (PeerMode::Start, Msg::Reverse) => {
                a.pop_visit();
            }

            _ => ()
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PeerMode {
    Start,
    Active,
    Closed
}

#[derive(Debug, PartialEq)]
pub enum LineMode {
    Basic,
    Browser
}


impl Debug for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state.addr.fmt(f)
    }
}

