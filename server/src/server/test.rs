use std::{io::Stdout, cell::Cell};
use assert_matches::*;

use super::{*, test_talker::TestTalk};

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
