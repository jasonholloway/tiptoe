use std::{rc::Rc, cell::{RefCell, Cell}, collections::HashMap, fmt::Debug};

use crate::common::*;

pub struct Roost<M, H> {
		peers: Vec<(Cell<M>, RR<H>)>,
    perches: HashMap<String, RR<H>>,
}

impl<M: Debug, H: Debug> Roost<M, H> {
		pub fn new() -> Roost<M, H> {
				Roost {
						peers: Vec::new(),
            perches: HashMap::new()
				}
		}

		pub fn add(&mut self, peer: (M, H)) -> () {
        let (m, h) = peer;
				self.peers.push((Cell::new(m), Rc::new(RefCell::new(h))));
		}

    pub fn find_perch(&self, tag: &Tag) -> Option<RR<H>> {
        self.perches.get(tag).map(|rc| rc.clone())
    }

    pub fn perch(&mut self, tag: Tag, peer: RR<H>) -> () {
        // println!("perching {}", tag);
        self.perches.insert(tag.to_string(), peer.clone());
    }

		pub fn iter(&self) -> impl Iterator<Item = &(Cell<M>, RR<H>)> {
        self.peers.iter()
		}

    pub fn clean(&mut self) {
        //go through peers removing all closed
        //...
    }
}

// impl<M: Debug, H: Debug> Debug for Roost<M, H> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let r: Vec<&(Cell<M>, RR<H>)> = self.peers.iter().map(|r| r.clone()).collect();
//         write!(f, "{:?}", &r)
//     }
// }

