use crate::{msg::{Msg, Cmd}, common::RR, visits::Step};

use core::fmt::Debug;
use std::{net::{TcpStream, SocketAddr}, collections::VecDeque};

use self::{input::{PeerInput, ReadResult}, output::PeerOutput};

mod input;
mod output;

pub struct Peer {
    pub input: PeerInput,
    pub output: PeerOutput,
    pub state: PeerState,
}

pub struct PeerState {
    mode: PeerMode,
    pub tag: String,
    addr: SocketAddr
}

impl Peer {
    pub fn new(addr: SocketAddr, stream: TcpStream) -> Peer {
        let stream2 = stream.try_clone().unwrap();
        Peer {
            input: PeerInput::new(stream),
            output: PeerOutput::new(stream2),
            state: PeerState {
                mode: PeerMode::Start,
                tag: String::new(),
                addr
            },
        }
    }

    pub fn pump(&mut self, pr: &RR<Peer>, cmds: &mut VecDeque<Cmd>) -> bool {
        match self.state.mode {
            PeerMode::Closed => false,
            _ => {
                match self.input.read() {
                    ReadResult::Yield(m) => {
                        if let Some(m2) = self.state.handle(pr, m) {
                            cmds.push_back(m2);
                        }
                        true
                    },
                    ReadResult::Continue => false,
                    ReadResult::Stop => {
                        self.state.mode = PeerMode::Closed;
                        true
                    }
                }
            }
        }
    }

    pub fn goto(&mut self, rf: &str) -> () {
        self.log("OUT", format!("goto {}", &rf));
        self.output.write(Msg::Goto(rf.to_string())).unwrap();
    }

    fn log<O: Debug>(&self, typ: &str, o: O) {
        // println!("{:?}@{} {} {:?}", self.state.addr, self.state.tag, typ, o)
    }
}

impl PeerState {
    pub fn handle(&mut self, rc: &RR<Peer>, msg: Msg) -> Option<Cmd> {
        match (&self.mode, msg) {
            (PeerMode::Start, Msg::Hello(new_tag)) => {
                self.tag = new_tag.to_string();
                self.mode = PeerMode::Active;
                Some(Cmd::Perch(new_tag, rc.clone()))
            }
            (PeerMode::Active, Msg::Stepped(from, to)) => {
                Some(Cmd::Stepped(Step { tag: self.tag.to_string(), from, to }))
            }

            (_, Msg::Reverse) => {
                Some(Cmd::Reverse)
            }
            (_, Msg::Hop) => {
                Some(Cmd::Hop)
            }
            (_, Msg::Clear) => {
                Some(Cmd::Clear)
            }

            _ => None
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PeerMode {
    Start,
    Active,
    Closed
}

impl Debug for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state.addr.fmt(f)
    }
}

