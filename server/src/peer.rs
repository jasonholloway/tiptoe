use crate::{msg::{Msg, Cmd}, common::RR, visits::Step};

use core::fmt::Debug;
use std::{net::{TcpStream, SocketAddr}, collections::VecDeque};

use self::{input::{PeerInput, ReadResult}, output::PeerOutput};

mod input;
mod output;

#[derive(Debug, PartialEq)]
pub enum PeerMode {
    Start,
    First,
    Active,
    Closed
}

impl Default for PeerMode {
    fn default() -> Self {
        PeerMode::Closed
    }
}

pub struct Peer {
    pub input: PeerInput,
    pub output: PeerOutput,
    pub tag: String,
    addr: SocketAddr
}

impl Peer {
    pub fn new(addr: SocketAddr, stream: TcpStream) -> Peer {
        let stream2 = stream.try_clone().unwrap();
        Peer {
            input: PeerInput::new(stream),
            output: PeerOutput::new(stream2),
            tag: String::new(),
            addr
        }
    }

    pub fn pump(&mut self, mode: PeerMode, pr: &RR<Peer>, cmds: &mut VecDeque<Cmd>) -> (PeerMode, bool) {
        match mode {
            PeerMode::Closed => (mode, false),
            _ => {
                match self.input.read() {
                    ReadResult::Yield(m) => (
                        self.handle(mode, pr, m, cmds),
                        true
                    ),
                    ReadResult::Continue => (mode, false),
                    ReadResult::Stop => (
                        PeerMode::Closed,
                        true
                    )
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

impl Peer {
    pub fn handle(&mut self, mode: PeerMode, rc: &RR<Peer>, msg: Msg, cmds: &mut VecDeque<Cmd>) -> PeerMode {
        match (mode, msg) {
            (PeerMode::Start, Msg::Hello(new_tag)) => {
                self.tag = new_tag.to_string();
                cmds.push_back(Cmd::Perch(new_tag, rc.clone()));
                PeerMode::First
            }

            (PeerMode::First, Msg::Stepped(from, to)) => {
                cmds.push_back(Cmd::Stepped(Step { tag: self.tag.to_string(), rf: from }));
                cmds.push_back(Cmd::Stepped(Step { tag: self.tag.to_string(), rf: to }));
                PeerMode::Active
            }


            (PeerMode::Active, Msg::Stepped(_, to)) => {
                cmds.push_back(Cmd::Stepped(Step { tag: self.tag.to_string(), rf: to }));
                PeerMode::Active
            }

            (s, Msg::Hop) => {
                cmds.push_back(Cmd::Hop);
                s
            }
            (s, Msg::Clear) => {
                cmds.push_back(Cmd::Clear);
                s
            }

            (s, _) => s
        }
    }
}

impl Debug for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.addr.fmt(f)
    }
}

