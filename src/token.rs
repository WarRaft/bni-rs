//! BNI's token kinds. BNI is a line-oriented data format with no keywords
//! or operators to speak of, so it doesn't use the shared `TokenKind`
//! vocabulary: its tokens are line roles (section name, key, value part).
//! They still flow through the shared [`Token`] / `TokenStream` plumbing.

use std::fmt;

pub use parser_rs::{ParseError, Token, TokenKindExt};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// `[` opening a section header.
    LBracket,
    /// The text between `[` and `]` (or the end of the line).
    SectionName(String),
    /// `]` closing a section header.
    RBracket,
    /// Everything before an item's first `=`, exactly as written.
    Key(String),
    /// The `=` of an item.
    Eq,
    /// A value part that is a whole integer, such as `42` or `-1`, and its
    /// raw text, so `-0` and `007` print as written.
    Int(i64, String),
    /// A value part that is a whole decimal, such as `0.5` or `-3.14`.
    /// The raw text is kept, so `0.00` prints as written.
    Real(String),
    /// Any other unquoted value part, trimmed.
    Text(String),
    /// `"..."`: the text between the quotes. `closed` is false when the
    /// line ends before the closing quote.
    Quoted { value: String, closed: bool },
    /// `,` between value fields.
    Comma,
    /// `// ...` to the end of the line.
    Comment,
    /// A line, or the rest of one, that fits no form: no `=`, or text
    /// after a section header's `]`.
    Junk(String),
    Newline,
    Eof,
}

impl TokenKindExt for Tok {
    fn is_eof(&self) -> bool {
        matches!(self, Tok::Eof)
    }
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::LBracket => write!(f, "'['"),
            Tok::SectionName(name) => write!(f, "section name '{name}'"),
            Tok::RBracket => write!(f, "']'"),
            Tok::Key(key) => write!(f, "key '{key}'"),
            Tok::Eq => write!(f, "'='"),
            Tok::Int(n, _) => write!(f, "integer {n}"),
            Tok::Real(r) => write!(f, "number {r}"),
            Tok::Text(t) => write!(f, "'{t}'"),
            Tok::Quoted { value, .. } => write!(f, "\"{value}\""),
            Tok::Comma => write!(f, "','"),
            Tok::Comment => write!(f, "comment"),
            Tok::Junk(t) => write!(f, "'{t}'"),
            Tok::Newline => write!(f, "end of line"),
            Tok::Eof => write!(f, "end of file"),
        }
    }
}
