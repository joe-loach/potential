use super::{
    Parser,
    SyntaxKind::{self, *},
};

enum Brackets {
    Round,
    Square,
}

pub(super) fn root(p: &mut Parser) {
    let m = p.start();
    while !p.at(Eof) {
        let m = p.start();
        let kind = match p.current() {
            OpenSquare => object(p),
            Ident if p.at_str("let") => shape(p),
            _ => {
                p.error("Expected to find a statement");
                p.bump_any();
                m.finish(p, Error);
                continue;
            }
        };
        p.expect(SemiColon);
        m.finish(p, kind);
    }
    m.finish(p, Root);
}

/// let label name (list)
fn shape(p: &mut Parser) -> SyntaxKind {
    assert!(p.at(Ident));
    p.bump_remap(Let);
    label(p);
    name(p);
    if p.at(OpenRound) {
        list(p, Brackets::Round);
    }
    Shape
}

/// [list]
fn object(p: &mut Parser) -> SyntaxKind {
    assert!(p.at(OpenSquare));
    list(p, Brackets::Square);
    Object
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
                value(p);
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
    let m = p.start();
    name(p);
    p.expect(Colon);
    m.finish(p, Label);
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

fn value(p: &mut Parser) {
    assert!(p.at(Literal));
    let m = p.start();
    p.bump(Literal);
    m.finish(p, Value);
}