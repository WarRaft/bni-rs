//! Groups the scanner's tokens into a [`Document`]. Every line is parsed
//! on its own, so a malformed line costs only itself: it's recorded as an
//! error and the parser moves to the next line.

use crate::ast::*;
use crate::lexer::tokenize;
use crate::token::{Tok, Token};
use parser_rs::TokenStream;

pub use parser_rs::ParseError;

pub struct Parser {
    tokens: TokenStream<Tok>,
    doc: Document,
}

impl Parser {
    pub fn parse_str(src: &str) -> Document {
        let mut p = Parser {
            tokens: TokenStream::new(tokenize(src)),
            doc: Document::default(),
        };
        while !p.tokens.at_eof() {
            p.line();
        }
        p.doc
    }

    /// One line, including its line break.
    fn line(&mut self) {
        match self.tokens.peek_kind() {
            Tok::LBracket => self.section(),
            Tok::Key(_) | Tok::Eq => {
                let item = self.item();
                match self.doc.sections.last_mut() {
                    Some(section) => section.items.push(item),
                    None => self.doc.preamble.push(item),
                }
            }
            Tok::Junk(_) => {
                let junk = self.tokens.advance();
                self.doc.errors.push(ParseError::at_token(
                    format!(
                        "expected '=' in {}: a line is a [section], a key=value item or a // comment",
                        junk.kind
                    ),
                    &junk,
                ));
            }
            _ => {}
        }
        self.end_line();
    }

    /// Skips to just past the line break (or to EOF). A trailing comment
    /// is skipped; text after a section header is reported.
    fn end_line(&mut self) {
        loop {
            let t = self.tokens.advance();
            match &t.kind {
                Tok::Newline | Tok::Eof => return,
                Tok::Junk(_) => self.doc.errors.push(ParseError::at_token(
                    format!("unexpected {} after the section header", t.kind),
                    &t,
                )),
                _ => {}
            }
        }
    }

    fn section(&mut self) {
        let open = self.tokens.advance();
        let name = match self.tokens.peek_kind().clone() {
            Tok::SectionName(text) => {
                let t = self.tokens.advance();
                Name { text, span: t.span() }
            }
            _ => Name {
                text: String::new(),
                span: open.span(),
            },
        };
        let closed = self.tokens.eat(&Tok::RBracket);
        self.doc.sections.push(Section {
            name,
            line: open.line,
            closed,
            items: Vec::new(),
        });
    }

    fn item(&mut self) -> Item {
        let key = match self.tokens.peek_kind().clone() {
            Tok::Key(text) => Some(Name {
                text,
                span: self.tokens.advance().span(),
            }),
            _ => None,
        };
        // The scanner emits `=` on every item line.
        let eq = self.tokens.advance();
        let mut fields = Vec::new();
        let mut current: Option<Field> = None;
        loop {
            let t = self.tokens.peek().clone();
            let kind = match &t.kind {
                Tok::Int(n) => ValueKind::Int(*n),
                Tok::Real(r) => ValueKind::Real(r.parse().unwrap_or(0.0)),
                Tok::Text(_) => ValueKind::Text,
                Tok::Quoted { closed, .. } => ValueKind::Quoted { closed: *closed },
                Tok::Comma => {
                    // A comma ends the current field, empty if nothing came.
                    fields.push(current.take().unwrap_or_else(|| empty_field(&t)));
                    self.tokens.advance();
                    // The field after the last comma exists even if empty.
                    current = None;
                    if !matches!(
                        self.tokens.peek_kind(),
                        Tok::Int(_) | Tok::Real(_) | Tok::Text(_) | Tok::Quoted { .. } | Tok::Comma
                    ) {
                        let after = self.tokens.prev().clone();
                        fields.push(empty_field_after(&after));
                    }
                    continue;
                }
                _ => break,
            };
            self.tokens.advance();
            let value = Value {
                kind,
                text: part_text(&t.kind),
                span: t.span(),
            };
            match &mut current {
                Some(field) => {
                    field.span = field.span.join(value.span);
                    field.parts.push(value);
                }
                None => {
                    current = Some(Field {
                        span: value.span,
                        parts: vec![value],
                    })
                }
            }
        }
        fields.extend(current);
        Item {
            key,
            line: eq.line,
            eq: eq.span(),
            fields,
        }
    }
}

fn part_text(kind: &Tok) -> String {
    match kind {
        Tok::Int(n) => n.to_string(),
        Tok::Real(s) | Tok::Text(s) | Tok::Quoted { value: s, .. } => s.clone(),
        _ => String::new(),
    }
}

/// An empty field just before the comma `t`.
fn empty_field(t: &Token<Tok>) -> Field {
    Field {
        parts: Vec::new(),
        span: parser_rs::Span::new(t.line, t.col, t.line, t.col),
    }
}

/// An empty field just after the comma `t`.
fn empty_field_after(t: &Token<Tok>) -> Field {
    Field {
        parts: Vec::new(),
        span: parser_rs::Span::new(t.line, t.end_col, t.line, t.end_col),
    }
}

#[cfg(test)]
#[path = "parser.test.rs"]
mod tests;
