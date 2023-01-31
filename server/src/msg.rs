pub type PeerTag = String;
pub type Ref = String;

#[derive(Debug)]
pub enum Msg {
    Hello(PeerTag),
    Visited(Ref),
    Revisit(Ref),

    Reverse,
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
        &["visited", raw_ref] => {
            Some(Msg::Visited(raw_ref.to_string()))
        }
        &["reverse"] => {
            Some(Msg::Reverse)
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
				Msg::Revisit(r) => {
						writeln!(w, "revisit {}", r)
				},
				_ => Ok(())
		}
}
