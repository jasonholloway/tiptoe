use crate::{BufReader, msg::{Msg, self}};
use core::fmt::Debug;
use std::{net::{TcpStream, SocketAddr}, io::{BufRead, ErrorKind}, str::from_utf8};


pub struct Peer {
    pub input: PeerInput,
    pub state: PeerState,
}

pub struct PeerInput {
    reader: BufReader<TcpStream>,
    buffer: Vec<u8>,
}

pub struct PeerState {
    mode: PeerMode,
    parse_mode: ParseMode,
    tag: String,
    addr: SocketAddr
}

pub enum PeerReadResult<'inp> {
    Line(&'inp str)
}

impl Peer {

    pub fn new(addr: SocketAddr, stream: TcpStream) -> Peer {
        Peer {
            input: PeerInput {
                reader: BufReader::new(stream),
                buffer: Vec::new()
            },
            state: PeerState {
                mode: PeerMode::Start,
                parse_mode: ParseMode::Basic,
                tag: String::new(),
                addr
            },
        }
    }

    pub fn read(&mut self) -> Option<Msg> {
        let s = &mut self.state;
        let input = &mut self.input;

        let msg = match (&s.mode, &s.parse_mode) {
            (PeerMode::Closed, _) => None,

            (_, ParseMode::Basic) => {
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

            (_, ParseMode::Browser) => {
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

    pub fn handle(&mut self, msg: Msg) -> Option<Msg> {
        let s = &mut self.state;

        match (&s.mode, msg) {
            (PeerMode::Start, Msg::Hello(new_tag, new_parse_mode)) => {
                s.tag = new_tag;
                s.parse_mode = new_parse_mode;
                s.mode = PeerMode::Active;
                None
            }
            (PeerMode::Active, Msg::Visited(r)) => {
                Some(Msg::VisitedTag(s.tag.to_string(), r))
            }
            (_, m) => Some(m)
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

#[derive(Debug, PartialEq)]
pub enum PeerMode {
    Start,
    Active,
    Closed
}

#[derive(Debug, PartialEq)]
pub enum ParseMode {
    Basic,
    Browser
}


impl Debug for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state.addr.fmt(f)
    }
}
