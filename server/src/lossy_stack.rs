use std::collections::VecDeque;

pub struct LossyStack<I> {
		pub deque: VecDeque<I>,
		cap: usize
}

impl<I: std::fmt::Debug> LossyStack<I> {
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
