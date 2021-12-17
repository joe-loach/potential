mod lexer;
mod parser;
mod text;

pub use text::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum SyntaxKind {
    /// A whitespace character
    Whitespace,
    /// // A comment
    Comment,
    /// An identifer
    Ident,
    /// A literal value
    Literal,

    /// :
    Colon,
    /// ;
    SemiColon,
    /// ,
    Comma,
    /// (
    OpenRound,
    /// )
    CloseRound,
    /// [
    OpenSquare,
    /// ]
    CloseSquare,

    Root,
    Stmt,
    Shape,
    Object,
    List,
    Param,
    Label,
    Name,
    Error,

    /// Unknown character to the lexer
    Unknown,
    /// End of file / stream
    Eof,
    /// Dead value, "doesn't exist"
    Tombstone,
}

use SyntaxKind::*;

pub fn parse(text: &str) {
    let tokens: Vec<_> = lexer::tokenise(text).collect();

    println!("Tokens");
    if false && cfg!(debug_assertions) {
        for it in &tokens {
            println!("{:?}", it);
        }
    }

    let mut parser = parser::Parser::new(
        tokens
            .iter()
            .filter_map(|it| {
                if !matches!(it.kind, Whitespace | Comment) {
                    Some(it.kind)
                } else {
                    None
                }
            })
            .collect(),
    );
    parser::grammar::root(&mut parser);
    let events = parser.finish();

    println!("Events");
    if cfg!(debug_assertions) {
        for it in &events {
            println!("{:?}", it);
        }
    }
}
