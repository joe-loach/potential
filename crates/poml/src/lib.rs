mod lexer;
pub mod parser;

use parser::Parse;

pub fn compile(text: &str) -> Result<Parse, Vec<String>> {
    let (text, mut errors) = lexer::lex(text);

    let (parse, parse_errors) = parser::parse(&text);
    errors.extend(parse_errors);
    let validation_errors = parser::ast::validate(&parse.syntax());
    errors.extend(validation_errors);

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(parse)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    ParamList,
    Param,
    Let,
    Label,
    Value,
    Name,
    Error,

    /// Unknown character to the lexer
    Unknown,
    /// End of file / stream
    Eof,
    /// Dead value, "doesn't exist"
    Tombstone,

    #[doc(hidden)]
    Last,
}

impl SyntaxKind {
    pub fn is_trivia(&self) -> bool {
        matches!(self, Whitespace | Comment)
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

impl From<u16> for SyntaxKind {
    fn from(raw: u16) -> Self {
        assert!(raw <= SyntaxKind::Last as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw) }
    }
}

use SyntaxKind::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Poml {}
impl rowan::Language for Poml {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        raw.0.into()
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<Poml>;
pub type SyntaxToken = rowan::SyntaxToken<Poml>;
pub type SyntaxElement = rowan::SyntaxElement<Poml>;
