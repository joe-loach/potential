use super::{ast::TreeBuilder, SyntaxKind};

#[derive(Debug)]
pub enum Event {
    Start { kind: SyntaxKind },
    Token { kind: SyntaxKind },
    Finish,
    Error { msg: Box<String> },
}

pub fn process(tb: &mut TreeBuilder, events: Vec<Event>) {
    for e in events {
        match e {
            Event::Start { kind } => {
                if kind != SyntaxKind::Tombstone {
                    tb.start_node(kind);
                }
            }
            Event::Token { kind } => tb.token(kind),
            Event::Finish => tb.finish_node(),
            Event::Error { msg } => tb.error(*msg),
        }
    }
}
