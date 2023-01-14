use std::{rc::Rc, cell::RefCell};


pub type Tag = String;

pub type RR<T> = Rc<RefCell<T>>;


