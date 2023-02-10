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

#[derive(PartialEq)]
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
            Starting => f.write_str("Starting"),
            Resting(_) => f.write_str("Resting"),
            Reaching(_,_) => f.write_str("Reaching"),
            Juggling(_,_) => f.write_str("Juggling"),
        }
    }
}


#[cfg(test)]
mod tests {
    use std::{sync::{Arc, Mutex}, io::Stderr, cell::Cell};
    use assert_matches::*;

    use crate::common::ReadResult;

    use super::*;

    #[test]
    fn hello_stepped() {
        let mut tt = TestRunner::new();

        let mut p = tt.peer("p1");
        p.say("hello moo");
        tt.step();
        tt.step();

        p.say("stepped a b");
        tt.step();

        assert_eq!(tt.state(), State::Resting(Step::new("moo", "b")));
    }

    #[test]
    fn juggle() {
        let mut tt = TestRunner::new();
        let mut p1 = tt.peer("p1");
        let mut p2 = tt.peer("p2");

        p1.say("hello p1");
        tt.step();
        tt.step();

        p1.say("stepped a b");
        tt.step();
        
        p1.say("stepped b c");
        tt.step();

        p2.say("juggle");
        tt.step();

        assert_matches!(tt.state(), Juggling(_, steps) => {
            assert_steps_eq(steps, vec!(
                ("p1", "b"),
                ("p1", "c")
            ));
        });
    }

    #[test]
    fn juggle2() {
        let mut tt = TestRunner::new();
        let mut p1 = tt.peer("p1");
        let mut p2 = tt.peer("p2");

        p1.say("hello p1");
        tt.step();
        tt.step();

        p1.say("stepped a b");
        tt.step();
        
        p1.say("stepped b c");
        tt.step();

        p1.say("stepped c d");
        tt.step();

        p2.say("juggle");
        tt.step();

        assert_matches!(tt.state(), Juggling(_, steps) => {
            assert_steps_eq(steps, vec!(
                ("p1", "c"),
                ("p1", "d")
            ));
        });
    }

    #[test]
    fn juggle3() {
        let mut tt = TestRunner::new();
        let mut p1 = tt.peer("p1");
        let mut p2 = tt.peer("p2");

        p1.say("hello p1");
        tt.step();
        tt.step();

        p1.say("stepped a b");
        tt.step();
        
        p1.say("stepped b c");
        tt.step();

        p2.say("juggle");
        tt.step();

        p1.say("stepped b d");
        tt.step();

        assert_matches!(tt.state(), Juggling(_, steps) => {
            assert_steps_eq(steps, vec!(
                ("p1", "c"),
                ("p1", "d")
            ));
        });
    }

    pub fn assert_steps_eq<'a, C: IntoIterator<Item=Step>>(steps: C, expected: Vec<(&str,&str)>) -> () {
        let actual_tups: Vec<(String, String)> = steps.into_iter()
            .map(|s| (s.tag, s.rf))
            .collect();

        let expected_tups: Vec<(String, String)> = expected.iter()
            .map(|(tag,rf)| (tag.to_string(), rf.to_string()))
            .collect();

        assert_eq!(actual_tups, expected_tups);
    }


    struct TestRunner {
        now: Instant,
        server: Server<TestTalk>,
        state_cell: Cell<Option<State>>,
        log: Stderr
    }

    impl TestRunner {
        pub fn new() -> TestRunner {
            let now = Instant::now();
            TestRunner {
                now,
                server: Server::new(now),
                state_cell: Cell::new(Some(Starting)),
                log: std::io::stderr()
            }
        }

        pub fn peer(&mut self, id: &str) -> TestTalk {
            let p = TestTalk::new();
            self.say(Cmd::Connect(Peer::new(id, p.clone())));
            p
        }

        pub fn state<'a>(&self) -> State {
            self.state_cell.take().unwrap()
        }

        pub fn say(&mut self, cmd: Cmd<TestTalk>) -> () {
            self.server.enqueue(cmd);
        }
        
        pub fn step(&mut self) -> () {
            self.now = self.now + Duration::from_secs(1);
            let (s2, _) = self.server.pump(self.state_cell.take().unwrap(), self.now, &mut self.log);
            self.state_cell.set(Some(s2));
        }
    }

    

    struct TestTalk {
        input: Arc<Mutex<VecDeque<String>>>,
        output: Arc<Mutex<VecDeque<String>>>
    }

    impl TestTalk {
        pub fn new() -> TestTalk {
            TestTalk {
                input: Arc::new(Mutex::new(VecDeque::new())),
                output: Arc::new(Mutex::new(VecDeque::new()))
            }
        }

        pub fn clone(&self) -> TestTalk {
            TestTalk {
                input: self.input.clone(),
                output: self.output.clone()
            }
        }

        pub fn say(&mut self, line: &str) -> () {
            self.input.lock().unwrap().push_back(line.to_string());
        }
    }

    impl Talk for TestTalk {
        fn read(&mut self) -> crate::common::ReadResult<String> {
            if let Some(popped) = self.input.lock().unwrap().pop_front() {
                ReadResult::Yield(popped)
            }
            else {
                ReadResult::Continue
            }
        }
    }

    impl std::fmt::Write for TestTalk {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            self.output.lock().unwrap().push_back(s.to_string());
            Ok(())
        }
    }

}
