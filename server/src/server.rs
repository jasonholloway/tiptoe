use std::{time::{Duration, Instant}, collections::VecDeque};

use crate::{roost::Roost, peer::{Peer, PeerMode}, lossy_stack::LossyStack, common::{Step, Cmd, Talk}};

use State::*;
use Cmd::*;

pub struct Server<S> {
    cmds: VecDeque<Cmd<S>>,
    roost: Roost<PeerMode, Peer<S>>,
    history: LossyStack<Step>,
    last_cleanup: Instant
}

pub enum State {
    Starting,
    Resting(Step),
    Reaching(Instant, VecDeque<Step>),
    Juggling(Instant, VecDeque<Step>)
}

impl<S: Talk> Server<S> {
    pub fn new(now: Instant) -> Server<S> {
        Server {
            roost: Roost::new(),
            history: LossyStack::new(128),
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

    fn tick(&mut self, state: State, now: &Instant) -> State {
        match state {
            Juggling(when, mut steps) if now.duration_since(when) > Duration::from_millis(700) => {
                if let Some(curr) = steps.pop_front() {
                    while let Some(popped) = steps.pop_back() {
                        self.history.push(popped);
                    }

                    Resting(curr)
                }
                else {
                    Starting
                }
            }
            Reaching(when, mut stack) if now.duration_since(when) > Duration::from_millis(700) => {
                let curr = stack.pop_back()
                    .expect("stack must always have at least one member");
                
                while let Some(popped) = stack.pop_back() {
                    self.history.push(popped);
                }
                
                Resting(curr)
            }
            s => s
        }
    }

    // so, on a hop, we cycle through our group
    // on a dredge the hop group is extended (the stack is added to)
    // initially the hop group is two
    // but through dredge we can go further back
    //

    fn handle(&mut self, state: State, cmd: Cmd<S>, now: &Instant) -> State {
        match (state, cmd) {
            (s, Connect(peer)) => {
                self.roost.add((PeerMode::Start, peer));
                s
            }

            (Starting, Stepped(step)) => {
                Resting(step)
            }
            (Resting(prev), Stepped(step)) => {
                self.history.push(prev);
                Resting(step)
            }
            (Juggling(_, mut steps), Stepped(step)) => {
                while let Some(popped) = steps.pop_back() {
                    self.history.push(popped);
                }
                Resting(step)
            }
            (Reaching(_, mut stack), Stepped(step)) => {
                while let Some(popped) = stack.pop_back() {
                    self.history.push(popped);
                }
                Resting(step)
            }



            (s@Starting, Juggle) => {
                s
            }
            (Resting(prev), Juggle) => {
                if let Some(step) = self.history.pop() {
                    self.goto(&step);
                    Juggling(*now, VecDeque::from([step,prev]))
                }
                else { Resting(prev) }
            }
            (Juggling(_, mut steps), Juggle) => {
                if let Some(prev) = steps.pop_front() {
                    steps.push_back(prev);
                }

                if let Some(curr) = steps.front() {
                    self.goto(&curr);
                }
                
                Juggling(*now, steps)
            }
            (s@Reaching(_,_), Juggle) => {
                s
            }
            

            (s@Starting, Reach) => {
                s
            }
            (Resting(prev), Reach) => {
                if let Some(step) = self.history.pop() {
                    self.goto(&step);
                    Reaching(*now, VecDeque::from([step,prev]))
                }
                else { Resting(prev) }
            }
            (Reaching(_, mut stack), Reach) => {
                if let Some(step) = self.history.pop() {
                    self.goto(&step);
                    stack.push_back(step);
                    Reaching(*now, stack)
                }
                else { Reaching(*now, stack) }
            }
            (Juggling(_, mut steps), Reach) => {
                if let Some(step) = self.history.pop() {
                    self.goto(&step);
                    //jumble occurs here I think todo
                    steps.push_back(step);
                    Reaching(*now, steps)
                }
                else {
                    Reaching(*now, steps)
                }
            }
            
            (s, Perch(tag, pr)) => {
                self.roost.perch(tag, pr);
                s
            }
            (s, Clear) => {
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


impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Starting => f.write_str("Start"),
            Resting(_) => f.write_str("AtRest"),
            Reaching(_,_) => f.write_str("Reaching"),
            Juggling(_,_) => f.write_str("Juggling"),
        }
    }
}








#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiptoe() {
        let now = Instant::now();
        // let _server: Server<TestStream> = Server::new(now);

        // server receives stream of peers and cmds
        // to test, a peer is announced into the system
        // and fomr there appends commands

        // server.pump(State::Starting, TcpListener::bind().unwrap(), now, nil); // 

        let r = dbg!("123");

        println!("{}", r);
    }

    struct TestStream {
        input: VecDeque<String>,
        output: VecDeque<String>
    }

    impl Talk for TestStream {
        fn read(&mut self) -> crate::common::ReadResult<String> {
            todo!()
        }
    }

    impl std::fmt::Write for TestStream {
        fn write_str(&mut self, _s: &str) -> std::fmt::Result {
            todo!()
        }
    }

}
