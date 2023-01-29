use crate::{msg::Msg, common::RR, traits::Actions};

use core::fmt::Debug;
use std::net::{TcpStream, SocketAddr};

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

    pub fn pump<A: Actions>(&mut self, act: &mut A, pr: &RR<Peer>) -> bool {
        match self.state.mode {
            PeerMode::Closed => false,
            _ => {
                match self.input.read() {
                    ReadResult::Yield(m) => {
                        self.state.handle(act, pr, m);
                        true
                    },
                    ReadResult::Continue => false,
                    ReadResult::Stop => {
                        self.log("Closed");
                        self.state.mode = PeerMode::Closed;
                        true
                    }
                }
            }
        }
    }

    fn log<O: Debug>(&self, o: O) {
        println!("{:?}: {:?}", self.state.addr, o)
    }
}

impl PeerState {
    pub fn handle<A: Actions>(&mut self, a: &mut A, rc: &RR<Peer>, msg: Msg) -> () {
        match (&self.mode, msg) {
            (PeerMode::Start, Msg::Hello(new_tag)) => {
                a.perch(&new_tag, rc.clone());
                self.tag = new_tag;
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

impl Debug for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state.addr.fmt(f)
    }
}

