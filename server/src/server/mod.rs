use std::{time::{Duration, Instant}, collections::VecDeque};

use crate::{roost::Roost, peer::{Peer, PeerMode}, common::{Step, Cmd, Talk}};

#[cfg(test)]
mod test;

#[cfg(test)]
mod test_talker;

use State::*;
use Cmd::*;

pub struct Server<S> {
    cmds: VecDeque<Cmd<S>>,
    roost: Roost<PeerMode, Peer<S>>,
    last_cleanup: Instant
}

#[derive(PartialEq)]
pub enum State {
    Starting,
    Resting(Vec<Step>),
    Juggling(Instant, VecDeque<Step>, Vec<Step>)
}

impl<S: Talk> Server<S> {
    pub fn new(now: Instant) -> Server<S> {
        Server {
            roost: Roost::new(),
            last_cleanup: now,
            cmds: VecDeque::new()
        }
    }

    pub fn enqueue(&mut self, cmd: Cmd<S>) -> () {
        self.cmds.push_back(cmd);
    }

    pub fn pump<W: std::io::Write>(&mut self, mut state: State, now: Instant, log: &mut W) -> (State, bool) {
        let mut work_done: bool = false;

        for (pmr, pr) in self.roost.iter() {
            let mut p = pr.borrow_mut();
            let m = pmr.take();
            
            let (m2, w) = p.pump(m, pr, &mut self.cmds, log);

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

    fn tick(&mut self, state: State, now: &Instant) -> State {
        match state {
            Juggling(when, steps, mut history)
                if now.duration_since(when) > Duration::from_millis(700) => {
                    history.append(&mut steps.into_iter().collect());
                    Resting(history)
                }
            s => s
        }
    }

    fn handle(&mut self, state: State, cmd: Cmd<S>, now: &Instant) -> State {
        match (state, cmd) {
            (s, Connect(peer)) => {
                self.roost.add((PeerMode::Start, peer));
                s
            }
            (s, Perch(tag, pr)) => {
                self.roost.perch(tag, pr);
                s
            }

            (Starting, Stepped(step)) => Resting(vec!(step)),
            (s@Starting, _) => s,

            (Resting(mut history), Stepped(step)) => {
                history.push(step);
                Resting(history)
            }
            (Resting(mut history), Juggle) => {
                if history.len() >= 2 {
                    let a = history.pop().unwrap();
                    let b = history.pop().unwrap();
                    self.goto(&b);
                    Juggling(*now, VecDeque::from(vec!(a, b)), history)
                }
                else {
                    Resting(history)
                }
            }

            (Juggling(_, active, mut history), Stepped(step)) => {
                history.append(&mut active.into_iter().collect());
                history.push(step);
                Resting(history)
            }
            (Juggling(_, mut active, history), Juggle) => {
                if let Some(prev) = active.pop_front() {
                    active.push_back(prev);
                }

                if let Some(curr) = active.front() {
                    self.goto(&curr);
                }
                
                Juggling(*now, active, history)
            }

            (_, Clear) => {
                todo!()
                // self.history.clear();
            }

            (Resting(_prev), Reach) => {
                todo!()
            }
            (Juggling(_, _active, _history), Reach) => {
                todo!()
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

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Starting => f.write_str("Starting"),
            Resting(_) => f.write_str("Resting"),
            Juggling(_,_,_) => f.write_str("Juggling"),
        }
    }
}


