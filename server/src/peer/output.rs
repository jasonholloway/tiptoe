use std::net::TcpStream;

use crate::msg::{Msg, self};

pub struct PeerOutput {
    writer: TcpStream,
		buffer: Vec<u8>
}

impl PeerOutput {
    pub fn new(stream: TcpStream) -> PeerOutput {
        PeerOutput {
            writer: stream,
            buffer: Vec::new()
        }
    }
    
    pub fn write(&mut self, msg: Msg) -> Result<(), std::io::Error> {
        println!("O {:?}", &self.buffer);
        msg::write(msg, &mut self.writer)
    }
}
