use std::{time::{Duration, Instant}, collections::VecDeque};

use crate::{roost::Roost, peer::{Peer, PeerMode}, common::{Step, Cmd, Talk}};

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


#[cfg(test)]
mod tests {
    use std::{sync::{Arc, Mutex}, io::Stdout, cell::Cell};
    use assert_matches::*;

    use crate::common::ReadResult;

    use super::*;

    #[test]
    fn hello_stepped() {
        let (mut tt, mut p) = setup1("p1");

        p.say("hello moo");
        tt.step();
        tt.step();

        p.say("stepped a b");
        tt.step();

        assert_matches!(tt.state(), Resting(steps) => {
            assert_steps_eq(steps, vec!(
                ("moo", "a"),
                ("moo", "b")
            ));
        });
    }

    #[test]
    fn juggle() {
        let (mut tt, mut p1, mut p2) = setup2("p1", "p2");

        p1.say("hello p1");
        tt.step();
        tt.step();

        p1.say("stepped a b");
        tt.step();
        
        p1.say("stepped b c");
        tt.step();

        p2.say("juggle");
        tt.step();

        assert_matches!(tt.state(), Juggling(_, active, history) => {
            assert_steps_eq(active, vec!(
                ("p1", "c"),
                ("p1", "b")
            ));
            assert_steps_eq(history, vec!(
                ("p1", "a")
            ));
        });
    }

    #[test]
    fn juggle_with_two() {
        let (mut tt, mut p1, mut p2) = setup2("p1", "p2");

        p1.say("hello p1");
        tt.step();
        tt.step();

        p1.say("stepped a b");
        tt.step();
        
        p1.say("stepped b c");
        tt.step();

        p2.say("juggle");
        tt.step();

        p2.say("juggle");
        tt.step();

        p2.say("juggle");
        tt.step();

        assert_eq!(p1.received(), &[
            "goto b",
            "goto c",
            "goto b"
        ]);

        assert_matches!(tt.state(), Juggling(_, active, history) => {
            assert_steps_eq(active, vec!(
                ("p1", "c"),
                ("p1", "b")
            ));

            assert_steps_eq(history, vec!(
                ("p1", "a")
            ));
        });
    }

    // #[test]
    // fn juggle2() {
    //     let (mut tt, mut p1, mut p2) = setup2("p1", "p2");

    //     p1.say("hello p1");
    //     tt.step();
    //     tt.step();

    //     p1.say("stepped a b");
    //     tt.step();
        
    //     p1.say("stepped b c");
    //     tt.step();

    //     p1.say("stepped c d");
    //     tt.step();

    //     p2.say("juggle");
    //     tt.step();

    //     assert_matches!(tt.state(), Juggling(_, active, history) => {
    //         assert_steps_eq(active, vec!(
    //             ("p1", "c"),
    //             ("p1", "d")
    //         ));
    //     });
    // }

    // #[test]
    // fn juggle3() {
    //     let (mut tt, mut p1, mut p2) = setup2("p1", "p2");

    //     p1.say("hello p1");
    //     tt.step();
    //     tt.step();

    //     p1.say("stepped a b");
    //     tt.step();
        
    //     p1.say("stepped b c");
    //     tt.step();

    //     p2.say("juggle");
    //     tt.step();

    //     p1.say("stepped b d");
    //     tt.step();

    //     assert_matches!(tt.state(), Juggling(_, active, _history) => {
    //         assert_steps_eq(active, vec!(
    //             ("p1", "c"),
    //             ("p1", "d")
    //         ));
    //     });
    // }

    fn setup1(p1_name: &str) -> (TestRunner, TestTalk) {
        let mut tt = TestRunner::new();
        let p1 = tt.peer(p1_name);
        (tt, p1)
    }

    fn setup2(p1_name: &str, p2_name: &str) -> (TestRunner, TestTalk, TestTalk) {
        let mut tt = TestRunner::new();
        let p1 = tt.peer(p1_name);
        let p2 = tt.peer(p2_name);
        (tt, p1, p2)
    }

    fn assert_steps_eq<'a, C: IntoIterator<Item=&'a Step>>(steps: C, expected: Vec<(&str,&str)>) -> () {
        let actual_tups: Vec<(String, String)> = steps.into_iter()
            .map(|s| (s.tag.to_string(), s.rf.to_string()))
            .collect();

        let expected_tups: Vec<(String, String)> = expected.iter()
            .map(|(tag,rf)| (tag.to_string(), rf.to_string()))
            .collect();

        assert_eq!(actual_tups, expected_tups);
    }


    struct TestRunner {
        now: Instant,
        server: Server<TestTalk>,
        state_cell: Cell<State>,
        log: Stdout
    }

    impl TestRunner {
        pub fn new() -> TestRunner {
            let now = Instant::now();
            TestRunner {
                now,
                server: Server::new(now),
                state_cell: Cell::new(Starting),
                log: std::io::stdout()
            }
        }

        pub fn peer(&mut self, id: &str) -> TestTalk {
            let p = TestTalk::new();
            self.say(Cmd::Connect(Peer::new(id, p.clone())));
            p
        }

        pub fn state<'a>(&'a mut self) -> &'a State {
            self.state_cell.get_mut()
        }

        pub fn say(&mut self, cmd: Cmd<TestTalk>) -> () {
            self.server.enqueue(cmd);
        }
        
        pub fn step(&mut self) -> () {
            self.now = self.now + Duration::from_secs(1);
            let (s2, _) = self.server.pump(self.state_cell.take(), self.now, &mut self.log);
            self.state_cell.set(s2);
        }
    }

    impl Default for State {
        fn default() -> Self {
            State::Starting
        }
    }

    

    struct TestTalk {
        input: Arc<Mutex<VecDeque<String>>>,
        output: Arc<Mutex<Vec<String>>>,
        buff: String
    }

    impl TestTalk {
        pub fn new() -> TestTalk {
            TestTalk {
                input: Arc::new(Mutex::new(VecDeque::new())),
                output: Arc::new(Mutex::new(Vec::new())),
                buff: "".to_string()
            }
        }

        pub fn say(&mut self, line: &str) -> () {
            self.input.lock().unwrap().push_back(line.to_string());
        }

        pub fn received<'a>(&'a self) -> Vec<String> {
            self.output.lock().ok().unwrap().iter()
                .map(|r| r.to_string())
                .collect()
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
            self.buff += s;

            while let Some((line, rest)) = self.buff.split_once('\n') {
                self.output.lock().unwrap().push(line.to_string());
                self.buff = rest.to_string();
            }
            
            Ok(())
        }
    }

    impl Clone for TestTalk {
        fn clone(&self) -> Self {
            TestTalk {
                input: self.input.clone(),
                output: self.output.clone(),
                buff: self.buff.to_string()
            }
        }
    }
}
