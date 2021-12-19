mod ast;
mod event;
mod grammar;

use crate::poml::SyntaxNode;
use ast::*;
use event::Event;
use rowan::GreenNode;

pub use ast::AstNode;

use super::{
    lexer::{LexedStr, Tokens, Trivia},
    SyntaxKind,
};

pub struct Parse {
    node: GreenNode,
    errors: Vec<String>,
}

impl Parse {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.node.clone())
    }

    pub fn root(&self) -> Root {
        Root::cast(self.syntax()).unwrap()
    }
}

pub fn parse(text: &LexedStr) -> Parse {
    let mut parser = Parser::new(text.tokens(Trivia::Skip));
    grammar::root(&mut parser);
    let events = parser.finish();
    let mut builder = ast::TreeBuilder::new(text.tokens(Trivia::Keep));
    event::process(&mut builder, events);
    let node = builder.finish();
    Parse {
        node,
        errors: Vec::new(),
    }
}

struct Parser<'t> {
    tokens: Tokens<'t>,
    pos: usize,
    events: Vec<Event>,
}

impl<'t> Parser<'t> {
    pub fn new(tokens: Tokens<'t>) -> Self {
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
        self.tokens.kind(self.pos + n)
    }

    fn at(&self, kind: SyntaxKind) -> bool {
        self.nth(0) == kind
    }

    fn at_str(&self, text: &str) -> bool {
        self.tokens.text(self.pos).map(|t| t == text).unwrap_or(false)
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

    fn bump_remap(&mut self, kind: SyntaxKind) {
        self.do_bump(kind);
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
