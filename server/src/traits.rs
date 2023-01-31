use crate::{common::{Tag, RR}, msg::Msg, peer::Peer};

pub trait Actions {
    fn perch(&mut self, tag: &Tag, cell: RR<Peer>) -> ();
    fn push_visit(&mut self, tag: Tag, reference: String) -> ();
    fn pop_visit(&mut self) -> ();
    fn clear_visits(&mut self) -> ();
}

pub trait HandleMsg {
    fn handle<A: Actions>(&mut self, a: &mut A, rc: &RR<Peer>, msg: Msg) -> Option<Msg>;
}
