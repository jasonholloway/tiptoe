use std::{rc::Rc, cell::RefCell, collections::HashMap, fmt::Debug};

use crate::common::*;

pub struct Roost<H> {
		peers: Vec<RR<H>>,
    perches: HashMap<String, RR<H>>,
}

impl<H: Debug> Roost<H> {
		pub fn new() -> Roost<H> {
				Roost {
						peers: Vec::new(),
            perches: HashMap::new()
				}
		}

		pub fn add(&mut self, peer: H) -> () {
				self.peers.push(Rc::new(RefCell::new(peer)))
		}

    pub fn find_perch(&self, tag: &Tag) -> Option<RR<H>> {
        self.perches.get(tag).map(|rc| rc.clone())
    }

    pub fn perch(&mut self, tag: Tag, peer: RR<H>) -> () {
        println!("perching {}", tag);
        self.perches.insert(tag.to_string(), peer.clone());
    }

		pub fn iter(&self) -> impl Iterator<Item = &RR<H>> {
        self.peers.iter()
		}

    pub fn clean(&mut self) {
        //go through peers removing all closed
        //...
    }
}

impl<H: Debug> Debug for Roost<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r: Vec<RR<H>> = self.peers.iter().map(|r| r.clone()).collect();
        write!(f, "{:?}", r)
    }
}

