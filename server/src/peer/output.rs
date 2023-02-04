use std::{net::TcpStream, io::Write};

pub struct PeerOutput {
    writer: TcpStream,
}

impl PeerOutput {
    pub fn new(stream: TcpStream) -> PeerOutput {
        PeerOutput {
            writer: stream,
        }
    }
    
    // pub fn write(&mut self, msg: Msg) -> Result<(), std::io::Error> {
    //     msg::write(msg, &mut self.writer).unwrap();
    // }
}

impl Write for PeerOutput {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let r = self.writer.write(buf);
        dbg!(r)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

