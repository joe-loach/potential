use super::{event::Event, SyntaxKind};

pub mod grammar {
    use super::SyntaxKind::*;
    use super::*;

    enum Brackets {
        Round,
        Square,
    }

    pub fn root(p: &mut Parser) {
        let m = p.start();
        while !p.at(Eof) {
            let m = p.start();
            label(p);
            match p.current() {
                OpenSquare => object(p),
                Ident => shape(p),
                _ => {
                    p.bump_any();
                    m.finish(p, Error);
                    continue;
                }
            }
            p.expect(SemiColon);
            m.finish(p, Stmt);
        }
        m.finish(p, Root);
    }

    /// Id
    fn shape(p: &mut Parser) {
        assert!(p.at(Ident));
        let m = p.start();
        name(p);
        if p.at(OpenRound) {
            list(p, Brackets::Round);
        }
        m.finish(p, Shape);
    }

    /// [Lit, Lit, Lit, Lit/Name]
    fn object(p: &mut Parser) {
        assert!(p.at(OpenSquare));
        let m = p.start();
        list(p, Brackets::Square);
        m.finish(p, Object);
    }

    fn list(p: &mut Parser, brack: Brackets) {
        fn param(p: &mut Parser) -> bool {
            match p.current() {
                Ident => {
                    let m = p.start();
                    name(p);
                    m.finish(p, Param);
                }
                Literal => {
                    let m = p.start();
                    p.bump(Literal);
                    m.finish(p, Param);
                }
                _ => return false,
            }
            true
        }

        let (open, close) = match brack {
            Brackets::Round => (OpenRound, CloseRound),
            Brackets::Square => (OpenSquare, CloseSquare),
        };
        assert!(p.at(open));
        let m = p.start();
        p.bump(open);
        while !p.at(Eof) && !p.at(close) {
            if !param(p) {
                break;
            }
            if !p.eat(Comma) {
                break;
            }
        }
        p.expect(close);
        m.finish(p, ParamList);
    }

    /// name:
    fn label(p: &mut Parser) {
        if let (Ident, Colon) = (p.nth(0), p.nth(1)) {
            let m = p.start();
            name(p);
            p.bump(Colon);
            m.finish(p, Label);
        }
    }

    fn name(p: &mut Parser) {
        if p.at(Ident) {
            let m = p.start();
            p.bump(Ident);
            m.finish(p, Name);
        } else {
            p.error("Expected to find a name");
        }
    }
}

pub struct Parser {
    tokens: Vec<SyntaxKind>,
    pos: usize,
    events: Vec<Event>,
}

impl Parser {
    pub fn new(tokens: Vec<SyntaxKind>) -> Self {
        Self {
            tokens,
            pos: 0,
            events: Vec::new(),
        }
    }

    pub fn finish(self) -> Vec<Event> {
        self.events
    }

    fn start(&mut self) -> Marker {
        let pos = self.events.len() as u32;
        self.events.push(Event::Start {
            kind: SyntaxKind::Tombstone,
        });
        Marker::new(pos)
    }

    fn current(&self) -> SyntaxKind {
        self.nth(0)
    }

    fn nth(&self, n: usize) -> SyntaxKind {
        self.tokens
            .get(self.pos + n)
            .copied()
            .unwrap_or(SyntaxKind::Eof)
    }

    fn at(&self, kind: SyntaxKind) -> bool {
        self.nth(0) == kind
    }

    fn eat(&mut self, kind: SyntaxKind) -> bool {
        if !self.at(kind) {
            return false;
        }
        self.do_bump(kind);
        true
    }

    fn bump(&mut self, kind: SyntaxKind) {
        assert!(self.eat(kind));
    }

    fn bump_any(&mut self) {
        self.eat(self.current());
    }

    fn error(&mut self, msg: impl Into<String>) {
        let msg = Box::new(msg.into());
        self.events.push(Event::Error { msg });
    }

    fn expect(&mut self, kind: SyntaxKind) -> bool {
        if self.eat(kind) {
            return true;
        }
        self.error(format!("expected {:?}", kind));
        false
    }

    fn do_bump(&mut self, kind: SyntaxKind) {
        self.pos += 1;
        self.events.push(Event::Token { kind })
    }
}

struct Marker {
    pos: u32,
}

impl Marker {
    pub fn new(pos: u32) -> Self {
        Self { pos }
    }

    pub fn finish(self, p: &mut Parser, kind: SyntaxKind) {
        let idx = self.pos as usize;
        match &mut p.events[idx] {
            Event::Start { kind: slot } => {
                *slot = kind;
            }
            _ => unreachable!(),
        }
        p.events.push(Event::Finish);
    }
}
