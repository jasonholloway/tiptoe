use crate::peer::ParseMode;

pub type PeerTag = String;
pub type Ref = String;

#[derive(Debug)]
pub enum Msg {
    Hello(PeerTag, ParseMode),
    Visited(Ref),
    VisitedTag(PeerTag, Ref),
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
                "basic" => Some(ParseMode::Basic),
                "browser" => Some(ParseMode::Browser),
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
