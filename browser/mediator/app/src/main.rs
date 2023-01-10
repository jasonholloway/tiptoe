use std::{net::{TcpListener, TcpStream, SocketAddr}, io::{BufReader, BufRead, ErrorKind}, str::*, time::Duration, collections::VecDeque, any};


#[derive(Debug, PartialEq)]
enum PeerMode {
    Start,
    Active
}

#[derive(Debug, PartialEq)]
enum ParseMode {
    Basic,
    Browser
}

type PeerName = String;
type Ref = String;

#[derive(Debug)]
enum Msg {
    Hello(PeerName, ParseMode),
    Selected(Ref)    
}



struct Peer<'b> {
    mode: PeerMode,
    parse_mode: ParseMode,
    name: String,
    addr: SocketAddr,
    reader: &'b BufReader<TcpStream>,
    buffer: &'b Vec<u8>,
}

impl core::fmt::Debug for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.addr.fmt(f)
    }
}

fn main() {
    let mut ref_log: VecDeque<Ref> = VecDeque::new();
    
    let listener = TcpListener::bind("127.0.0.1:17878").unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut peers: Vec<Peer> = Vec::new();
    let mut closable_peers: Vec<usize> = Vec::new();

    loop {
        let mut work_done: bool = false;
        
        match listener.accept() {
            Ok((stream, address)) => {
                stream.set_nonblocking(true).unwrap();
                peers.push(Peer {
                    mode: PeerMode::Start,
                    parse_mode: ParseMode::Basic,
                    name: String::new(),
                    addr: address,
                    reader: BufReader::new(stream),
                    buffer: Vec::new()
                });
                work_done = true;
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => {
                panic!("Unexpected connect error {e:?}")
            }
        };

        for (i, p) in peers.iter_mut().enumerate() {
            match read_peer(p) {
                Some(PeerReadResult::Line(line)) => {
                    let parsed_msg = parse_msg(p, line);
                    // println!("{0:?}: {1:?} {2:?} {3:?}", p.addr, p.buffer, line, &parsed_msg);
                    // p.buffer.clear();

                    // parsed_msg.map(|m| handle_msg(&mut ref_log, *p, m));

                    work_done = true;
                }
                Some(PeerReadResult::Close) => {
                    closable_peers.push(i);
                    println!("{0:?} END", p.addr)
                }
                None => ()
            }
        }

        for i in closable_peers.iter() {
            peers.remove(*i);
        }

        closable_peers.clear();

        if !work_done {
            std::thread::sleep(Duration::from_millis(100));
        }
    };
}

enum PeerReadResult<'inp> {
    Line(&'inp str),
    Close
}


fn read_peer<'p>(peer: &'p mut Peer) -> Option<PeerReadResult<'p>> {
    match peer.parse_mode {
        ParseMode::Basic => {
            match peer.reader.read_until(b'\n', &mut peer.buffer) {
                Ok(0) => Some(PeerReadResult::Close),
                Ok(_) => {
                    let line = from_utf8(&peer.buffer).unwrap();
                    Some(PeerReadResult::Line(line))
                }, 
                Err(e) if e.kind() == ErrorKind::WouldBlock => None,
                Err(e) => {
                    println!("Unexpected read error {e:?}");
                    Some(PeerReadResult::Close)
                }
            }
        }
        _ => None
        // ParseMode::Browser => peer.reader.read_until(b';', &mut peer.buffer),
    }
}

fn parse_msg(peer: &mut Peer, raw_line: &str) -> Option<Msg> {
    let words = match peer.mode {
        _ => {
            raw_line
                .split(|c: char| c.is_whitespace() || c == ';')
                .filter(|w| !w.is_empty())
                .collect::<Vec<_>>()
        }
    };

    let parsed = match words.as_slice() {
        &["hello", name, raw_mode] => {
            let parsed_mode = match raw_mode {
                "basic" => Some(ParseMode::Basic),
                "browser" => Some(ParseMode::Browser),
                _ => None
            };

            parsed_mode.map(|m| Msg::Hello(name.to_string(), m))
        }
        &["selected", raw_ref] => {
            Some(Msg::Selected(raw_ref.to_string()))
        }
        _ => None
    };

    if parsed.is_none() {
        println!("Unparsable line {}", raw_line);
    }

    parsed
}

fn handle_msg(ref_log: &mut VecDeque<Ref>, peer: Peer, msg: Msg) -> () {
    match (&peer.mode, msg) {
        (PeerMode::Start, Msg::Hello(new_name, new_parse_mode)) => {
            peer.name = new_name;
            peer.parse_mode = new_parse_mode;
            peer.mode = PeerMode::Active;
        }

        (PeerMode::Active, Msg::Selected(r)) => {
            println!("ref {}", r);
            //todo should half the log if too big here
            ref_log.push_front(r);
        }

        _ => ()
    }
}

// each connection that comes in:
// - either it is in 'browser mode' determined by
//
//
// - the browser executes a script
// - the script then redirects directly to a socket
//   but with a special initial message to opt into 'browser mode'
//
//
// originally the idea was to append all to a file
// movements would then involve going through this log
// 
// the app here then is just a logging machine
// although - we also want to be able to distribute commands
// so each connector will announce itself to us, with a name and a mode
//






//we want to write to a log
//but then as we go back through the log
//each time we go back it appends to the log
//
//a succession of movements is its own log
//a following movement goes either on to the movement log or the main one
//so, do we have two files? I think we may
//
//main.log and movement.log
//or maybe the latter can be all in memory
//
//we hear of a new state
//if the tail of the movement log wasn't that long back, append to movement log
//
//but wait a sec, this is only when moving from within tiptoe
//
//external movements are heard, and what then? they don't go on the movement log at all
//they can be happily appended to the main log
//
//when moving, there needs to be a time-limited mask of input events based on the movements we have in fact made
