use std::{io::{BufReader, ErrorKind, BufRead}, net::TcpStream, str::from_utf8};
use std::io::Write;

use crate::common::{ReadResult,Talk};


pub struct TcpTalker {
    input: BufReader<TcpStream>,
    output: TcpStream,
    buffer: Vec<u8>,
    active: bool
}

impl TcpTalker {
    pub fn new(stream: TcpStream) -> TcpTalker {
        TcpTalker {
            input: BufReader::new(stream.try_clone().unwrap()),
            output: stream,
            buffer: Vec::new(),
            active: true
        }
    }
}

impl Talk for TcpTalker {
    fn read(&mut self) -> ReadResult<String> {
        if !self.active {
            ReadResult::Stop
        }
        else {
            match self.input.read_until(b'\n', &mut self.buffer) {
                Ok(0) => {
                    self.active = false;
                    ReadResult::Stop
                }
                Ok(_) => {
                    let result = self.buffer
                        .split_last()
                        .and_then(|(_,l)| from_utf8(l).ok())
                        .map(|s| ReadResult::Yield(s.trim().to_string()))
                        .unwrap_or(ReadResult::Continue);

                    self.buffer.clear();

                    result
                }, 
                Err(e) if e.kind() == ErrorKind::WouldBlock => ReadResult::Continue,
                Err(e) => {
                    println!("Unexpected read error {e:?}");
                    self.active = false;
                    ReadResult::Stop
                }
            }
        }
    }
}

impl std::fmt::Write for TcpTalker {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        writeln!(self.output, "{}", s).expect("Failed to write to tcp");
        Ok(())
    }
}
