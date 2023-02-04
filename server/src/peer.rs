use crate::common::{Cmd,RR,Step};

use core::fmt::Debug;
use std::io::Write;
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
                    ReadResult::Yield(line) => (
                        self.handle(mode, pr, line, cmds),
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
        writeln!(self.output, "goto {}", rf).unwrap();
        self.output.flush().unwrap();
    }
}

impl Peer {
    pub fn handle(&mut self, mode: PeerMode, rc: &RR<Peer>, line: String, cmds: &mut VecDeque<Cmd>) -> PeerMode {
        match (mode, Self::digest(&line).as_slice()) {

            (PeerMode::Start, &["hello", new_tag]) => {
                self.tag = new_tag.to_string();
                cmds.push_back(Cmd::Perch(new_tag.to_string(), rc.clone()));
                PeerMode::First
            }

            (PeerMode::First, &["stepped", from, to]) => {
                cmds.push_back(Cmd::Stepped(Step { tag: self.tag.to_string(), rf: from.to_string() }));
                cmds.push_back(Cmd::Stepped(Step { tag: self.tag.to_string(), rf: to.to_string() }));
                PeerMode::Active
            }


            (PeerMode::Active, &["stepped", _, to]) => {
                cmds.push_back(Cmd::Stepped(Step { tag: self.tag.to_string(), rf: to.to_string() }));
                PeerMode::Active
            }

            (s, &["hop"]) => {
                cmds.push_back(Cmd::Hop);
                s
            }
            (s, &["clear"]) => {
                cmds.push_back(Cmd::Clear);
                s
            }

            (s, _) => s
        }
    }

    fn digest(line: &String) -> Vec<&str> {
        line.trim().split_whitespace().collect::<Vec<_>>()
    }
}

impl Debug for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.addr.fmt(f)
    }
}

