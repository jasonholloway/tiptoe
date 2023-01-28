use crate::peer::LineMode;

pub type PeerTag = String;
pub type Ref = String;

#[derive(Debug)]
pub enum Msg {
    Hello(PeerTag, LineMode),
    Visited(Ref),
    Reverse,
    Revisit(Ref)
}

pub fn try_parse(raw_line: &str) -> Option<Msg> {
    let words = raw_line
        .split(|c: char| c.is_whitespace() || c == ';')
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>();

    let parsed = match words.as_slice() {
        &["hello", tag, raw_mode] => {
            let parsed_mode = match raw_mode {
                "basic" => Some(LineMode::Basic),
                "browser" => Some(LineMode::Browser),
                _ => None
            };

            parsed_mode.map(|m| Msg::Hello(tag.to_string(), m))
        }
        &["visited", raw_ref] => {
            Some(Msg::Visited(raw_ref.to_string()))
        }
        &["reverse"] => {
            Some(Msg::Reverse)
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
						write!(w, "\"revisit {}\"", r)
				},
				_ => Ok(())
		}
}
