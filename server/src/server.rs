use std::{net::TcpListener, io::ErrorKind, time::Duration};

use crate::{roost::Roost, visits::{Visits, Visit}, common::{Tag, RR}, traits::Actions, peer::Peer};

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

            let prs: Vec<RR<Peer>> = self.roost.iter()
                .map(|r| r.clone())
                .collect();

            for pr in prs {
                let mut p = pr.borrow_mut();
                work_done = p.pump(self, &pr);
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
        for v in self.visits.pop().and_then(|_| self.visits.peek()) {
            for rc in self.roost.find_perch(&v.tag) {
                let mut p = rc.borrow_mut();
                p.revisit(&v.reference);
            }
        }
    }
}

