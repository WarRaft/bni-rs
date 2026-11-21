//! Strings in BNI values, for text forms of binary files that hold any
//! string. BNI strings have no escapes and no line breaks, so what a quoted
//! part can't hold goes between quoted parts, unquoted, as an escape:
//! `"Line one"\n"Line two"`. [`quote`] writes that, [`unquote`] reads it.
//! The escapes are `\n`, `\r`, `\t` and `\u{...}` (a character's hex
//! code); `"` is `\u{22}`.

use parser_rs::{ParseError, Span};

use crate::ast::{Field, ValueKind};

/// `s` as a value that [`unquote`] reads back: it always starts with a
/// quoted part.
pub fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    let mut quoted = true;
    for c in s.chars() {
        let escape = match c {
            '\n' => Some("\\n".to_string()),
            '\r' => Some("\\r".to_string()),
            '\t' => Some("\\t".to_string()),
            '"' => Some("\\u{22}".to_string()),
            c if c.is_control() => Some(format!("\\u{{{:X}}}", c as u32)),
            _ => None,
        };
        match (escape, quoted) {
            (Some(e), true) => {
                out += "\"";
                out += &e;
                quoted = false;
            }
            (Some(e), false) => out += &e,
            (None, false) => {
                out.push('"');
                out.push(c);
                quoted = true;
            }
            (None, true) => out.push(c),
        }
    }
    if quoted {
        out.push('"');
    }
    out
}

/// The string a field holds: its quoted parts as written, the unquoted
/// parts between them escapes.
pub fn unquote(field: &Field) -> Result<String, ParseError> {
    let mut out = String::new();
    for part in &field.parts {
        match part.kind {
            ValueKind::Quoted { closed: false } => {
                return Err(error(part.span, "string isn't closed: `\"` missing"));
            }
            ValueKind::Quoted { closed: true } => out += &part.text,
            _ => unescape(&part.text, &mut out).map_err(|m| error(part.span, m))?,
        }
    }
    Ok(out)
}

fn unescape(text: &str, out: &mut String) -> Result<(), String> {
    const FORM: &str = "between quoted parts only escapes: \\n \\r \\t or \\u{...}";
    let mut rest = text;
    while !rest.is_empty() {
        let after = rest.strip_prefix('\\').ok_or(FORM)?;
        let (c, next) = match after.as_bytes().first() {
            Some(b'n') => ('\n', &after[1..]),
            Some(b'r') => ('\r', &after[1..]),
            Some(b't') => ('\t', &after[1..]),
            Some(b'u') => {
                let (hex, next) = after[1..]
                    .strip_prefix('{')
                    .and_then(|h| h.split_once('}'))
                    .ok_or(FORM)?;
                let c = u32::from_str_radix(hex, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .filter(|&c| c != '\0');
                (c.ok_or("\\u{...} takes a character's hex code, not 0")?, next)
            }
            _ => return Err(FORM.into()),
        };
        out.push(c);
        rest = next;
    }
    Ok(())
}

fn error(span: Span, message: impl Into<String>) -> ParseError {
    let end_col = if span.end_line == span.start_line {
        span.end_col.max(span.start_col + 1)
    } else {
        span.start_col + 1
    };
    ParseError {
        message: message.into(),
        line: span.start_line,
        col: span.start_col,
        end_col,
    }
}

#[cfg(test)]
#[path = "string.test.rs"]
mod tests;
