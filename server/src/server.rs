use std::{net::TcpListener, io::ErrorKind, time::Duration, collections::VecDeque, fmt::Write};

use crate::{roost::Roost, visits::{Visits, LossyStack, Step}, common::RR, peer::Peer, msg::Cmd};

pub struct Server {
    steps: LossyStack<Step>,
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

    pub fn pump<W: std::io::Write>(&mut self, listener: TcpListener, log: &mut W) {
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
                writeln!(log, "\t\t\t{:?}", cmd).unwrap();
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
            Cmd::Hop => {
                if let Some(step) = self.steps.pop().map(|s| s.flip()) {
                    for rc in self.roost.find_perch(&step.tag) {
                        let mut p = rc.borrow_mut();
                        p.goto(&step.to);
                    }

                    self.steps.push(step);
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
