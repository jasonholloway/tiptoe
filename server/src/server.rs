use std::{net::TcpListener, io::ErrorKind, time::{Duration, Instant}, collections::VecDeque};

use crate::{roost::Roost, visits::Step, peer::{Peer, PeerMode}, msg::Cmd, lossy_stack::LossyStack};

pub struct Server {
    cmds: VecDeque<Cmd>,
    roost: Roost<PeerMode, Peer>,
    history: LossyStack<Step>,
    last_cleanup: Instant
}

pub enum State {
    Start,
    AtRest(Step),
    Moving(Instant, VecDeque<Step>)
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Start => f.write_str("Start"),
            State::AtRest(_) => f.write_str("AtRest"),
            State::Moving(_,_) => f.write_str("Moving"),
        }
    }
}


impl Server {
    pub fn new(now: Instant) -> Server {
        Server {
            roost: Roost::new(),
            history: LossyStack::new(128),
            last_cleanup: now,
            cmds: VecDeque::new()
        }
    }

    pub fn pump<W: std::io::Write>(&mut self, mut state: State, listener: &mut TcpListener, now: Instant, log: &mut W) -> (State, bool) {
        let mut work_done: bool = false;

        work_done |= self.accept_peers(listener);

        for (pmr, pr) in self.roost.iter() {
            let mut p = pr.borrow_mut();
            let m = pmr.take();
            
            let (m2, w) = p.pump(m, pr, &mut self.cmds);

            pmr.set(m2);
            work_done |= w;
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

    fn accept_peers(&mut self, listener: &mut TcpListener) -> bool {
        match listener.accept() {
            Ok((stream, address)) => {
                stream.set_nonblocking(true).unwrap();
                self.cmds.push_back(Cmd::Connect(Peer::new(address, stream)));
                true
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => false,
            Err(e) => {
                panic!("Unexpected connect error {e:?}")
            }
        }
    }

    fn tick(&mut self, state: State, now: &Instant) -> State {
        match state {
            State::Moving(when, mut stack) if now.duration_since(when) > Duration::from_millis(700) => {
                let curr = stack.pop_back()
                    .expect("stack must always have at least one member");
                
                while let Some(popped) = stack.pop_back() {
                    self.history.push(popped);
                }
                
                State::AtRest(curr)
            }
            s => s
        }
    }

    fn handle(&mut self, state: State, cmd: Cmd, now: &Instant) -> State {
        match (state, cmd) {
            (s, Cmd::Connect(peer)) => {
                self.roost.add((PeerMode::Start, peer));
                s
            }

            (State::Start, Cmd::Stepped(step)) => {
                State::AtRest(step)
            }
            (State::AtRest(curr), Cmd::Stepped(step)) => {
                self.history.push(curr);
                State::AtRest(step)
            }
            (State::Moving(_, mut stack), Cmd::Stepped(step)) => {
                stack.push_back(step);
                State::Moving(*now, stack)
            }

            (s@State::Start, Cmd::Hop) => {
                s
            }
            (State::AtRest(curr), Cmd::Hop) => {
                if let Some(step) = self.history.pop() {
                    self.goto(&step);
                    State::Moving(*now, VecDeque::from([curr,step]))
                }
                else { State::AtRest(curr) }
            }
            (State::Moving(_, mut stack), Cmd::Hop) => {
                if let Some(step) = self.history.pop() {
                    self.goto(&step);
                    stack.push_back(step);
                    State::Moving(*now, stack)
                }
                else { State::Moving(*now, stack) }
            }
            
            (s, Cmd::Perch(tag, pr)) => {
                self.roost.perch(tag, pr);
                s
            }
            (s, Cmd::Clear) => {
                self.history.clear();
                s
            }
        }
    }

    fn goto(&mut self, step: &Step) {
        if let Some(rc) = self.roost.find_perch(&step.tag) {
            let mut p = rc.borrow_mut();
            p.goto(&step.rf);
        }
    }
}
