use super::SyntaxKind;

#[derive(Debug)]
pub enum Event {
    Start { kind: SyntaxKind },
    Token { kind: SyntaxKind },
    Finish,
    Error { msg: Box<String> },
}

impl Event {
    pub fn tombstone() -> Event {
        Event::Start {
            kind: SyntaxKind::Tombstone,
        }
    }
}

pub fn process(events: Vec<Event>) {
    for e in events {
        match e {
            Event::Start { kind } => {
                if kind != SyntaxKind::Tombstone {
                    // start_node
                }
            }
            Event::Finish => todo!(),
            Event::Token { kind } => todo!(),
            Event::Error { msg } => todo!(),
        }
    }
}
