use std::{net::TcpListener, io::ErrorKind, time::Duration, collections::VecDeque};

use crate::{roost::Roost, visits::{Visits, Visit}, common::{Tag, RR}, traits::Actions, peer::Peer, msg::Msg};

pub struct Server {
    visits: Visits,
    roost: Roost<Peer>
}

impl Server {
    pub fn new() -> Server {
        Server {
            visits: Visits::new(128),
            roost: Roost::new()
        }
    }

    pub fn pump(&mut self, listener: TcpListener) {
        let mut buff: VecDeque<(RR<Peer>,Msg)> = VecDeque::new();
        
        loop {
            let mut work_done: bool = false;

            match listener.accept() {
                Ok((stream, address)) => {
                    stream.set_nonblocking(true).unwrap();
                    self.roost.add(Peer::new(address, stream));
                    work_done = true;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(e) => {
                    panic!("Unexpected connect error {e:?}")
                }
            };

            for pr in self.roost.iter() {
                let mut p = pr.borrow_mut();
                for m in p.read() {
                    buff.push_back((pr.clone(),m));
                }
            }

            while let Some((pr,m)) = buff.pop_front() {
                let mut p = pr.borrow_mut();
                p.state.handle(self, &pr, m);
                work_done = true;
            }

            let cleanup_due = true;
            if cleanup_due {
                //todo, should clean every 100 loops or similar
                self.roost.clean();
            }

            if !work_done {
                std::thread::sleep(Duration::from_millis(30));
            }
        };
    }
}

impl Actions for Server {
    fn perch(&mut self, tag: &Tag, peer: RR<Peer>) -> () {
        self.roost.perch(tag, peer);
    }

    fn push_visit(&mut self, tag: Tag, reference: String) -> () {
        self.visits.push(Visit { tag: tag.to_string(), reference: reference.to_string() });
    }

    fn pop_visit(&mut self) -> () {
        for v in self.visits.pop() {
            println!("Now back to {:?}", v);
            for _perch in self.roost.find_perch(&v.tag) {
                // println!("Found perch {}", &v.tag);
                // let mut other = perch.borrow_mut();
                // other.handle(&mut self.roost, blah, Msg::Revisit(v.reference.to_string()));
            }
        }
    }
}

