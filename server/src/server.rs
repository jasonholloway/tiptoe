use std::{net::TcpListener, io::ErrorKind, time::{Duration, Instant}, collections::VecDeque};

use crate::{roost::Roost, visits::{LossyStack, Step}, peer::Peer, msg::Cmd};

pub struct Server {
    cmds: VecDeque<Cmd>,
    roost: Roost<Peer>,
    history: LossyStack<Step>,
    stack: LossyStack<Step>,
    last_cleanup: Instant
}

pub enum State {
    AtRest,
    Moving(Instant, Step)
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::AtRest => f.write_str("AtRest"),
            State::Moving(_,_) => f.write_str("Moving"),
        }
    }
}


impl Server {
    pub fn new(now: Instant) -> Server {
        Server {
            roost: Roost::new(),
            history: LossyStack::new(128),
            stack: LossyStack::new(128),
            last_cleanup: now,
            cmds: VecDeque::new()
        }
    }

    pub fn pump<W: std::io::Write>(&mut self, mut state: State, listener: &mut TcpListener, now: Instant, log: &mut W) -> (State, bool) {
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
            work_done |= p.pump(&pr, &mut self.cmds);
        }

        state = self.tick(state, &now);

        while let Some(cmd) = self.cmds.pop_front() {
            writeln!(log, "\t\t\t{:?} {:?}", state, cmd).unwrap();
            state = self.handle(state, cmd, &now);
            work_done = true;
        }

        if now.duration_since(self.last_cleanup) > Duration::from_secs(10) {
            self.roost.clean(); //TODO prune the roost of old peers
            self.last_cleanup = now;
        }

        (state, work_done)
    }

    fn tick(&mut self, state: State, now: &Instant) -> State {
        match state {
            State::Moving(when, step) if now.duration_since(when) > Duration::from_millis(800) => {
                while let Some(popped) = self.stack.pop() {
                    self.history.push(popped);
                }

                self.history.push(step);
                
                State::AtRest
            }
            s => s
        }
    }

    fn handle(&mut self, state: State, cmd: Cmd, now: &Instant) -> State {
        match (state, cmd) {

            (State::AtRest, Cmd::Hop) => {
                if let Some(step) = self.history.pop() {
                    for rc in self.roost.find_perch(&step.tag) {
                        let mut p = rc.borrow_mut();
                        p.goto(&step.from);
                    }

                    State::Moving(*now, step.flip())
                }
                else { State::AtRest }
            }

            (State::Moving(_, prev_step), Cmd::Hop) => {
                if let Some(step) = self.history.pop() {
                    for rc in self.roost.find_perch(&step.tag) {
                        let mut p = rc.borrow_mut();
                        p.goto(&step.from);
                    }

                    self.stack.push(prev_step);
                    State::Moving(*now, step.flip())
                }

                else { State::Moving(*now, prev_step) }
            }
            
            (s, Cmd::Perch(tag, pr)) => {
                self.roost.perch(tag, pr);
                s
            }
            (s, Cmd::Stepped(step)) => {
                self.history.push(step);
                s
            }
            (s, Cmd::Clear) => {
                self.history.clear();
                s
            }
        }
    }
}
