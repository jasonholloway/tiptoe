
pub struct PeerInput {
    reader: BufReader<TcpStream>,
    buffer: Vec<u8>,
    active: bool
}

pub enum ReadResult {
    Yield(Msg),
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
    
    pub fn read(&mut self) -> ReadResult {
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
                    let msg = self.buffer
                        .split_last()
                        .and_then(|(_,l)| from_utf8(l).ok())
                        .and_then(msg::try_parse);

                    self.buffer.clear();

                    match msg {
                        Some(m) => ReadResult::Yield(m),
                        None => ReadResult::Continue
                    }
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
