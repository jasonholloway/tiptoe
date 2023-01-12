use std::{net::{TcpListener, TcpStream, SocketAddr}, io::{BufReader, BufRead, ErrorKind}, str::*, time::Duration, collections::VecDeque};


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

type PeerTag = String;
type Ref = String;

#[derive(Debug)]
enum Msg {
    Hello(PeerTag, ParseMode),
    Visited(Ref)    
}


#[derive(Debug)]
struct Visit {
    tag: String,
    reference: Ref
}


struct Peer {
    input: PeerInput,
    state: PeerState,
}

struct PeerInput {
    reader: BufReader<TcpStream>,
    buffer: Vec<u8>,
}

struct PeerState {
    mode: PeerMode,
    parse_mode: ParseMode,
    tag: String,
    addr: SocketAddr
}



impl core::fmt::Debug for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state.addr.fmt(f)
    }
}

fn main() {
    let mut visits: VecDeque<Visit> = VecDeque::new();
    
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
                    input: PeerInput {
                        reader: BufReader::new(stream),
                        buffer: Vec::new()
                    },
                    state: PeerState {
                        mode: PeerMode::Start,
                        parse_mode: ParseMode::Basic,
                        tag: String::new(),
                        addr: address,
                    },
                });
                work_done = true;
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => {
                panic!("Unexpected connect error {e:?}")
            }
        };

        for (i, p) in peers.iter_mut().enumerate() {
            match read_peer(&mut p.input, &p.state) {
                Some(PeerReadResult::Line(line)) => {
                    let parsed_msg = parse_msg(&p.state, line);
                    println!("{0:?}: {1:?} {2:?}", p.state.addr, line, parsed_msg);

                    parsed_msg.map(|m| handle_msg(&mut visits, &mut p.state, m));

                    p.input.buffer.clear();
                    work_done = true;
                }
                Some(PeerReadResult::Close) => {
                    closable_peers.push(i);
                    println!("{0:?} END", p.state.addr)
                }
                None => ()
            }
        }

        for i in closable_peers.iter() {
            peers.remove(*i);
        }

        closable_peers.clear();

        if !work_done {
            std::thread::sleep(Duration::from_millis(30));
        }
    };
}

fn read_peer<'i>(input: &'i mut PeerInput, peer: &PeerState) -> Option<PeerReadResult<'i>> {
    match peer.parse_mode {
        ParseMode::Basic => {
            match input.reader.read_until(b'\n', &mut input.buffer) {
                Ok(0) => Some(PeerReadResult::Close),
                Ok(_) => {
                    input.buffer
                        .split_last()
                        .map(|(_,l)| from_utf8(l).unwrap())
                        .map(|s| PeerReadResult::Line(s))
                }, 
                Err(e) if e.kind() == ErrorKind::WouldBlock => None,
                Err(e) => {
                    println!("Unexpected read error {e:?}");
                    Some(PeerReadResult::Close)
                }
            }
        }
        ParseMode::Browser => {
            match input.reader.read_until(b';', &mut input.buffer) {
                Ok(0) => Some(PeerReadResult::Close),
                Ok(_) => {
                    input.buffer
                        .split_last()
                        .map(|(_,l)| from_utf8(l).unwrap())
                        .and_then(|s| s.split_once('*'))
                        .map(|(_,s)| PeerReadResult::Line(s))
                }, 
                Err(e) if e.kind() == ErrorKind::WouldBlock => None,
                Err(e) => {
                    println!("Unexpected read error {e:?}");
                    Some(PeerReadResult::Close)
                }
            }
        }
    }
}

enum PeerReadResult<'inp> {
    Line(&'inp str),
    Close
}


fn parse_msg(peer: &PeerState, raw_line: &str) -> Option<Msg> {
    let words = match peer.mode {
        _ => {
            raw_line
                .split(|c: char| c.is_whitespace() || c == ';')
                .filter(|w| !w.is_empty())
                .collect::<Vec<_>>()
        }
    };

    let parsed = match words.as_slice() {
        &["hello", tag, raw_mode] => {
            let parsed_mode = match raw_mode {
                "basic" => Some(ParseMode::Basic),
                "browser" => Some(ParseMode::Browser),
                _ => None
            };

            parsed_mode.map(|m| Msg::Hello(tag.to_string(), m))
        }
        &["visited", raw_ref] => {
            Some(Msg::Visited(raw_ref.to_string()))
        }
        _ => None
    };

    if parsed.is_none() {
        println!("Unparsable line {}", raw_line);
    }

    parsed
}

fn handle_msg(ref_log: &mut VecDeque<Visit>, peer: &mut PeerState, msg: Msg) -> () {
    match (&peer.mode, msg) {
        (PeerMode::Start, Msg::Hello(new_tag, new_parse_mode)) => {
            peer.tag = new_tag;
            peer.parse_mode = new_parse_mode;
            peer.mode = PeerMode::Active;
        }

        (PeerMode::Active, Msg::Visited(r)) => {
            println!("ref {}", r);
            //todo should halve the log if too big here
            ref_log.push_front(Visit {
                tag: peer.tag.clone(),
                reference: r
            });
        }

        _ => ()
    }
}

//have index of peers
//instead of simple vec
//each tag is given an integer handle
//
//but - what about untagged???



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
