use std::collections::VecDeque;
use std::fmt::Debug;

use crate::msg::{PeerTag, Ref};



#[derive(Debug)]
pub struct Step {
    pub tag: PeerTag,
		pub from: Ref,
		pub to: Ref
}

impl Step {
		pub fn flip(&self) -> Step {
				Step {
						tag: self.tag.to_string(),
						from: self.to.to_string(),
						to: self.from.to_string()
				}
		}
}






pub type Visits = LossyStack<Step>;



pub struct LossyStack<I> {
		pub deque: VecDeque<I>,
		cap: usize
}

impl<I: Debug> LossyStack<I> {
		pub fn new(capacity: usize) -> LossyStack<I> {
				LossyStack {
						deque: VecDeque::with_capacity(capacity),
						cap: capacity
				}
		}

		pub fn push(&mut self, item: I) -> () {
				if self.cap - self.deque.len() <= 1 {
						self.deque.truncate(self.cap / 2);
				}

        // println!("pushed {:?}", &item);
				
				self.deque.push_front(item);
		}

		pub fn pop(&mut self) -> Option<I> {
				self.deque.pop_front()
		}

		pub fn peek(&self) -> Option<&I> {
				self.deque.front()
		}

		pub fn clear(&mut self) -> () {
				self.deque.clear();
		}
}

/*
but now we need to switch windows as well
Visited(ref) needs to include window id in its ref

then we can tell sway to move to the window id

that means we need to hook into sway changes first and foremost
just a series of changes

SWAY HOOS PLEASE


*/
