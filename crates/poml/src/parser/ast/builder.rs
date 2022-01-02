use crate::SyntaxKind;

use std::mem;

use crate::lexer::Tokens;

pub struct TreeBuilder<'t> {
    tokens: Tokens<'t>,
    inner: rowan::GreenNodeBuilder<'static>,
    errors: Vec<String>,
    state: State,
    pos: usize,
}

enum State {
    PendingStart,
    Normal,
    PendingFinish,
}

impl<'t> TreeBuilder<'t> {
    pub fn new(tokens: Tokens<'t>) -> Self {
        Self {
            tokens,
            errors: Vec::new(),
            inner: rowan::GreenNodeBuilder::new(),
            state: State::PendingStart,
            pos: 0,
        }
    }

    pub fn finish(mut self) -> (rowan::GreenNode, Vec<String>) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingFinish => {
                self.eat_trivias();
                self.inner.finish_node();
            }
            State::PendingStart | State::Normal => unreachable!(),
        }

        (self.inner.finish(), self.errors)
    }

    pub fn token(&mut self, kind: SyntaxKind) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingStart => unreachable!(),
            State::PendingFinish => self.inner.finish_node(),
            State::Normal => (),
        }
        self.eat_trivias();
        self.do_token(kind);
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingStart => (),
            State::PendingFinish => self.inner.finish_node(),
            State::Normal => (),
        }
        self.inner.start_node(kind.into());
    }

    pub fn finish_node(&mut self) {
        match mem::replace(&mut self.state, State::PendingFinish) {
            State::PendingStart => unreachable!(),
            State::PendingFinish => self.inner.finish_node(),
            State::Normal => (),
        }
    }

    pub fn error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn eat_trivias(&mut self) {
        while self.pos < self.tokens.len() {
            let kind = self.tokens.kind(self.pos);
            if !kind.is_trivia() {
                break;
            }
            self.do_token(kind);
        }
    }

    pub fn do_token(&mut self, kind: SyntaxKind) {
        let text = self.tokens.text(self.pos).unwrap();
        self.pos += 1;
        self.inner.token(kind.into(), text);
    }
}
