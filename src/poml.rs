mod lexer;
mod text;

pub use text::*;

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
    let _tokens: Vec<_> = lexer::tokenise(text).collect();
}
