use crate::server::Talk;
use std::{sync::{Arc, Mutex}, collections::VecDeque};

use crate::common::ReadResult;


pub struct TestTalk {
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
    fn read(&mut self) -> ReadResult<String> {
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
