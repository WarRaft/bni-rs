//! The BNI document: sections of `key=value` items. Comments are trivia
//! and aren't kept.

pub use parser_rs::{ColumnEncoding, ParseError, Span};

/// A parsed file. Parsing never fails as a whole: a malformed line is
/// skipped and recorded in `errors`, and every other line still parses.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    /// Items before the first section header.
    pub preamble: Vec<Item>,
    pub sections: Vec<Section>,
    /// One error per malformed line, in source order.
    pub errors: Vec<ParseError>,
}

impl Document {
    /// The first section named `name` (names compare exactly).
    pub fn section(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.name.text == name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    /// The name between the brackets, as written. Empty for `[]`, with a
    /// span at the `[`.
    pub name: Name,
    pub line: usize,
    /// Whether the header has its closing `]`.
    pub closed: bool,
    pub items: Vec<Item>,
}

impl Section {
    /// Every item with key `key`, in order. Keys may repeat: in
    /// `TriggerStrings.txt` the first is a name, the next ones a format
    /// and a hint.
    pub fn get<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a Item> + 'a {
        self.items
            .iter()
            .filter(move |i| i.key.as_ref().is_some_and(|k| k.text == key))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

/// `key=value,value,...`. Both sides are optional.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// Everything before the first `=`, exactly as written, so leading
    /// and trailing spaces are part of it. `None` only when the line
    /// starts with `=`.
    pub key: Option<Name>,
    pub line: usize,
    /// The `=`.
    pub eq: Span,
    /// The comma-separated fields; empty for `key=`.
    pub fields: Vec<Field>,
}

impl Item {
    /// The value as text: each field's text, joined with `,`.
    pub fn value_text(&self) -> String {
        self.fields.iter().map(Field::text).collect::<Vec<_>>().join(",")
    }
}

/// One comma-separated field. Usually one part; `a"b"` has two; `a,,b`
/// has an empty field in the middle.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub parts: Vec<Value>,
    /// The parts' extent, or a zero-width span where an empty field is.
    pub span: Span,
}

impl Field {
    /// The field's text: parts concatenated, quoted ones without quotes.
    pub fn text(&self) -> String {
        self.parts.iter().map(|p| p.text.as_str()).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub kind: ValueKind,
    /// The text as written; for a quoted string, without its quotes.
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    Int(i64),
    Real(f64),
    Text,
    Quoted { closed: bool },
}
