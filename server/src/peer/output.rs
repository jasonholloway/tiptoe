use std::{net::TcpStream, io::Write};

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
        println!("OUT {:?}", &msg);
        msg::write(msg, &mut self.writer).unwrap();
        self.writer.flush()
    }
}
