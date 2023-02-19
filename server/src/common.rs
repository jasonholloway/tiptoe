use std::{rc::Rc, cell::RefCell};

use crate::peer::Peer;

pub type Tag = String;

pub type RR<T> = Rc<RefCell<T>>;

pub type PeerTag = String;

pub type Ref = String;

pub enum Cmd<S> {
    Connect(Peer<S>),
    Perch(PeerTag, RR<Peer<S>>),
    Stepped(Step),
    Reach,
    Juggle,
    Clear
}

#[derive(Debug, PartialEq)]
pub struct Step {
    pub tag: PeerTag,
		pub rf: Ref,
}

impl Step {
    // pub fn new(tag: &str, rf: &str) -> Step {
    //     Step {
    //         tag: tag.to_string(),
    //         rf: rf.to_string()
    //     }
    // }
}

impl<S> std::fmt::Debug for Cmd<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          Cmd::Connect(_) => write!(f, "Connect"),
          Cmd::Perch(tag, _) => write!(f, "Perch  {}", tag),
          Cmd::Stepped(step) => write!(f, "Stepped {:?}", step),
          Cmd::Reach => write!(f, "Reach"),
          Cmd::Juggle => write!(f, "Juggle"),
          Cmd::Clear => write!(f, "Clear")
        }
    }
}


pub trait Talk where Self: std::fmt::Write {
    fn read(&mut self) -> ReadResult<String>;
}

pub enum ReadResult<R> {
    Yield(R),
    Continue,
    Stop
}
