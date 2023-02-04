use std::{io::{BufReader, ErrorKind, BufRead}, net::TcpStream, str::from_utf8};

pub struct PeerInput {
    reader: BufReader<TcpStream>,
    buffer: Vec<u8>,
    active: bool
}

pub enum ReadResult<R> {
    Yield(R),
    Continue,
    Stop
}

impl PeerInput {
    pub fn new(stream: TcpStream) -> PeerInput {
        PeerInput {
            reader: BufReader::new(stream),
            buffer: Vec::new(),
            active: true
        }
    }

    pub fn read(&mut self) -> ReadResult<String> {
        if !self.active {
            ReadResult:: Stop
        }
        else {
            match self.reader.read_until(b'\n', &mut self.buffer) {
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
