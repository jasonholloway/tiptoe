use crate::{common::RR, peer::Peer, visits::Step};

pub type PeerTag = String;
pub type Ref = String;

#[derive(Debug)]
pub enum Msg {
    Hello(PeerTag),
    Stepped(Ref,Ref),
    Goto(Ref),

    Reverse,
    Hop,
    Clear
}

#[derive(Debug)]
pub enum Cmd {
    Connect(Peer),
    Perch(PeerTag, RR<Peer>),
    Stepped(Step),
    Hop,
    Clear
}

pub fn try_parse(raw_line: &str) -> Option<Msg> {
    let words = raw_line
        .split(|c: char| c.is_whitespace() || c == ';')
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>();

    let parsed = match words.as_slice() {
        &["hello", tag] => {
            Some(Msg::Hello(tag.to_string()))
        }
        &["stepped", from_ref, to_ref] => {
            Some(Msg::Stepped(from_ref.to_string(), to_ref.to_string()))
        }
        &["hop"] => {
            Some(Msg::Hop)
        }
        &["clear"] => {
            Some(Msg::Clear)
        }
        _ => None
    };

    if parsed.is_none() {
        println!("Unparsable line {}", raw_line);
    }

    parsed
}

pub fn write<W: std::io::Write>(m: Msg, w: &mut W) -> Result<(), std::io::Error> {
		match m {
				Msg::Goto(r) => {
						writeln!(w, "goto {}", r)
				},
				_ => Ok(())
		}
}
