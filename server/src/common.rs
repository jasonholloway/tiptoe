use std::{rc::Rc, cell::RefCell};

use crate::peer::Peer;

pub type Tag = String;

pub type RR<T> = Rc<RefCell<T>>;

pub type PeerTag = String;

pub type Ref = String;

#[derive(Debug)]
pub enum Cmd {
    Connect(Peer),
    Perch(PeerTag, RR<Peer>),
    Stepped(Step),
    Hop,
    Clear
}

#[derive(Debug)]
pub struct Step {
    pub tag: PeerTag,
		pub rf: Ref,
}

