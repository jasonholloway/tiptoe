use std::time::Duration;
use std::thread;
use std::io::{BufReader, ErrorKind};
use std::net::TcpListener;

mod common;
mod msg;
mod peer;
mod roost;
mod visits;

use msg::*;
use peer::*;
use roost::*;
use visits::*;

fn main() {
		let mut visits = Visits::new(128);
    
    let listener = TcpListener::bind("127.0.0.1:17878").unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut peers: Roost = Roost::new();

    loop {
        let mut work_done: bool = false;
        
        match listener.accept() {
            Ok((stream, address)) => {
                stream.set_nonblocking(true).unwrap();
                peers.add(Peer::new(address, stream));
                work_done = true;
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => {
                panic!("Unexpected connect error {e:?}")
            }
        };

        for pr in peers.iter() {
            let p = &mut pr.borrow_mut();
            
            for m1 in p.read() {
                for m2 in p.handle(m1) {
                    handle(&mut visits, &peers, m2);
                }
                work_done = true;
            }
        }

        let cleanup_due = true;
        if cleanup_due {
            //todo, should clean every 100 loops or similar
            peers.clean();
        }

        if !work_done {
            thread::sleep(Duration::from_millis(30));
        }
    };
}

fn handle(visits: &mut Visits, peers: &Roost, msg: Msg) -> () {
    match msg {
        Msg::VisitedTag(tag, reference) => {
            visits.push(Visit { tag: tag.to_string(), reference: reference.to_string() });
        }

        Msg::Reverse => {
            for v in visits.pop() {
                for perch in peers.find_perch(&v.tag) {
                    let mut peer = perch.borrow_mut();
                    println!("Now back to {:?}", v);
                    peer.handle(Msg::Revisit(v.reference.to_string()));
                }
            }
        }

        _ => ()
    }
}
