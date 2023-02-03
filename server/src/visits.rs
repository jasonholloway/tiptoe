use std::fmt::Debug;

use crate::msg::{PeerTag, Ref};

#[derive(Debug)]
pub struct Step {
    pub tag: PeerTag,
		pub rf: Ref,
}

