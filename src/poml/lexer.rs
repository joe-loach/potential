use super::SyntaxKind;

use rowan::TextRange;

#[derive(Debug)]
pub struct Token {
    pub kind: SyntaxKind,
    pub range: TextRange,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trivia {
    Keep,
    Skip,
}

pub struct Tokens<'t> {
    pub source: &'t str,
    tokens: &'t [Token],
    idxs: Vec<usize>,
}

impl<'t> Tokens<'t> {
    pub fn get(&self, n: usize) -> Option<&Token> {
        let i = *self.idxs.get(n)?;
        Some(&self.tokens[i])
    }

    pub fn kind(&self, n: usize) -> SyntaxKind {
        self.get(n).map(|t| t.kind).unwrap_or(SyntaxKind::Eof)
    }

    pub fn text(&self, n: usize) -> Option<&str> {
        let t = self.get(n)?;
        Some(&self.source[t.range])
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }
}

pub struct LexedStr<'t> {
    source: &'t str,
    tokens: Vec<Token>,
}

impl<'t> LexedStr<'t> {
    pub const fn new(source: &'t str, tokens: Vec<Token>) -> Self {
        Self { source, tokens }
    }

    pub fn tokens(&self, trivia: Trivia) -> Tokens {
        let idxs: Vec<usize> = if trivia == Trivia::Skip {
            self.tokens
                .iter()
                .enumerate()
                .filter_map(
                    |(i, tok)| {
                        if !tok.kind.is_trivia() {
                            Some(i)
                        } else {
                            None
                        }
                    },
                )
                .collect()
        } else {
            (0..self.tokens.len()).collect()
        };

        Tokens {
            source: self.source,
            tokens: &self.tokens,
            idxs,
        }
    }
}

pub fn lex(mut text: &str) -> LexedStr {
    let source = text;
    let tokens = std::iter::from_fn({
        let mut pos = 0.into();
        move || {
            if text.is_empty() {
                // no more text to lex
                return None;
            }
            // make the next token
            let (kind, len) = {
                let mut c = Cursor::new(text);
                (c.next(), c.ate.into())
            };
            let tok = Token {
                kind,
                range: TextRange::at(pos, len),
            };
            // remove the lexed text
            text = &text[len.try_into().unwrap()..];
            pos += len;
            Some(tok)
        }
    })
    .collect();

    LexedStr::new(source, tokens)
}

use std::{iter::Peekable, str::Chars};

pub(crate) struct Cursor<'a> {
    chars: Peekable<Chars<'a>>,
    ate: u32,
}

impl<'a> Cursor<'a> {
    fn new(text: &'a str) -> Self {
        assert!(!text.is_empty());
        Self {
            chars: text.chars().peekable(),
            ate: 0,
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn eat(&mut self) -> Option<char> {
        self.ate += 1;
        self.chars.next()
    }

    fn eat_while<P>(&mut self, pred: P) -> u32
    where
        P: Fn(char) -> bool,
    {
        let mut ate = 0;
        while let Some(c) = self.peek() {
            if pred(c) {
                self.eat();
                ate += 1;
            } else {
                break;
            }
        }
        ate
    }

    fn next(&mut self) -> SyntaxKind {
        use SyntaxKind::*;

        let first = self.eat().expect("text is not empty");

        match first {
            // Whitespace
            c if whitespace(c) => {
                self.eat_while(whitespace);
                Whitespace
            }
            // Comment
            '/' => {
                if let Some('/') = self.peek() {
                    self.eat(); // eat the 2nd slash
                    self.eat_while(|c| c != '\n');
                    Comment
                } else {
                    Unknown
                }
            }
            // Ident
            c if ident(c) => {
                self.eat_while(ident);
                Ident
            }
            // Literal
            c @ ('-' | '.' | '0'..='9') => self.literal(c),

            // Single chars
            ':' => Colon,
            ';' => SemiColon,
            ',' => Comma,
            '(' => OpenRound,
            ')' => CloseRound,
            '[' => OpenSquare,
            ']' => CloseSquare,
            _ => Unknown,
        }
    }

    /// Possible literal "configurations"
    /// 0
    /// 0.
    /// 0.0
    /// .0
    /// -0
    /// -0.0
    fn literal(&mut self, first: char) -> SyntaxKind {
        if let '-' | '.' = first {
            // check that the next character is also a number
            if !self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                return SyntaxKind::Unknown;
            }
        }
        // keep eating the number
        self.eat_while(|c| matches!(c, '0'..='9' | '_'));
        if !(first == '.') {
            // there might be a decimal point
            if let Some('.') = self.peek() {
                self.eat(); // eat the '.'
                self.eat_while(|c| matches!(c, '0'..='9' | '_'));
            }
        }
        SyntaxKind::Literal
    }
}

fn whitespace(c: char) -> bool {
    c.is_ascii_whitespace()
}

fn ident(c: char) -> bool {
    c.is_ascii_alphabetic() || matches!(c, '_')
}
