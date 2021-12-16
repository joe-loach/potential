mod lexer;
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

    /// Unknown character to the lexer
    Unknown,
}

pub fn parse(text: &str) {
    let tokens: Vec<_> = lexer::tokenise(text).collect();

    if cfg!(debug_assertions) {
        for it in &tokens {
            println!("{:?}", it);
        }
    }
}
