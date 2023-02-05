use crate::common::{Cmd,ReadResult,RR,Step,Talk};

use core::fmt::Debug;
use std::collections::VecDeque;

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

pub struct Peer<S> {
    pub talk: S,
    pub tag: String,
    id: String
}

impl<S: Talk> Peer<S> {
    pub fn new(id: &str, talk: S) -> Peer<S> {
        Peer {
            talk,
            tag: String::new(),
            id: id.to_string()
        }
    }

    pub fn pump<W: std::io::Write>(&mut self, mode: PeerMode, pr: &RR<Peer<S>>, cmds: &mut VecDeque<Cmd<S>>, log: &mut W) -> (PeerMode, bool) {
        match mode {
            PeerMode::Closed => (mode, false),
            _ => {
                match self.talk.read() {
                    ReadResult::Yield(line) => {
                        writeln!(log, "Read {}", &line).unwrap();
                        (
                            self.handle(mode, pr, line, cmds),
                            true
                        )
                    },
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
        writeln!(self.talk, "goto {}", rf).unwrap();
    }
}

impl<S> Peer<S> {
    pub fn handle(&mut self, mode: PeerMode, rc: &RR<Peer<S>>, line: String, cmds: &mut VecDeque<Cmd<S>>) -> PeerMode {
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

            (s, &["juggle"]) => {
                cmds.push_back(Cmd::Juggle);
                s
            }
            (s, &["reach"]) => {
                cmds.push_back(Cmd::Reach);
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

impl<S> Debug for Peer<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

