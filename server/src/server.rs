use std::{net::TcpListener, io::ErrorKind, time::Duration, collections::VecDeque};

use crate::{roost::Roost, visits::Visits, common::RR, peer::Peer, msg::Cmd};

pub struct Server {
    steps: Visits,
    roost: Roost<Peer>,
    cmds: VecDeque<Cmd>
}

impl Server {
    pub fn new() -> Server {
        Server {
            steps: Visits::new(128),
            roost: Roost::new(),
            cmds: VecDeque::new()
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
                work_done |= p.pump(&pr, &mut self.cmds);
            }

            while let Some(cmd) = self.cmds.pop_front() {
                work_done |= self.handle(cmd);
            }

            let cleanup_due = true;
            if cleanup_due {
                //todo, should clean every 100 loops or similar
                //any closed peers in roost: get rid
                self.roost.clean();
            }

            if !work_done {
                std::thread::sleep(Duration::from_millis(30));
            }
        };
    }

    fn handle(&mut self, cmd: Cmd) -> bool {
        println!("CMD {:?}", &cmd);
        match cmd {
            Cmd::Perch(tag, pr) => {
                self.roost.perch(tag, pr);
                true
            }
            Cmd::Stepped(step) => {
                self.steps.push(step);
                true
            }
            Cmd::Reverse => {
                for step in self.steps.pop() {
                    for rc in self.roost.find_perch(&step.tag) {
                        let mut p = rc.borrow_mut();
                        p.goto(&step.from);
                    }
                }
                true
            }
            Cmd::Clear => {
                self.steps.clear();
                true
            }
        }
    }
}
