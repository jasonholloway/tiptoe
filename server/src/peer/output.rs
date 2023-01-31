use std::{net::TcpStream, io::Write};

use crate::msg::{Msg, self};

pub struct PeerOutput {
    writer: TcpStream,
}

impl PeerOutput {
    pub fn new(stream: TcpStream) -> PeerOutput {
        PeerOutput {
            writer: stream,
        }
    }
    
    pub fn write(&mut self, msg: Msg) -> Result<(), std::io::Error> {
        msg::write(msg, &mut self.writer).unwrap();
        self.writer.flush()
    }
}
