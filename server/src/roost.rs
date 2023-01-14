use std::{rc::Rc, cell::RefCell, collections::HashMap};

use crate::common::*;
use crate::peer::*;

pub struct Roost {
		peers: Vec<RR<Peer>>,
    perches: HashMap<Tag, RR<Peer>>
}

impl Roost {
		pub fn new() -> Roost {
				Roost {
						peers: Vec::new(),
            perches: HashMap::new()
				}
		}

		pub fn add(&mut self, peer: Peer) -> () {
				self.peers.push(Rc::new(RefCell::new(peer)))
		}

    pub fn find_perch(&self, tag: &Tag) -> Option<RR<Peer>> {
        self.perches.get(tag).map(|rc| rc.clone())
    }

    pub fn tag_peer(&mut self, peer: RR<Peer>, tag: &Tag) -> () {
        self.perches.insert(*tag, peer.clone());
    }

		pub fn iter(&mut self) -> impl Iterator<Item = &RR<Peer>> {
        self.peers.iter()
		}

    pub fn clean(&mut self) {
        //go through peers removing all closed
        //...
    }
}


