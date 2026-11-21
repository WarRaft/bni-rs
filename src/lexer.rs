//! The BNI line scanner. BNI values are free text, with spaces and
//! anything but commas, quotes and `//` in them, and what a character means
//! depends on where in the line it stands (a `[` opens a section only at the
//! start). So BNI has its own scanner instead of a `LexSpec`. It is still
//! the **only** lexer: the parser, the semantic highlighter and the HTML
//! viewer all read its tokens.
//!
//! The scanner never fails. A line that fits no form becomes a
//! [`Tok::Junk`] token, and the parser reports it.

use crate::token::{Tok, Token};

/// Scans `src` into tokens: each line's tokens, then [`Tok::Newline`]
/// (none after the last line), then [`Tok::Eof`]. `\r\n` endings and a
/// leading byte-order mark are accepted.
pub fn tokenize(src: &str) -> Vec<Token<Tok>> {
    let src = src.strip_prefix('\u{FEFF}').unwrap_or(src);
    let mut out = Vec::new();
    let (mut line, mut end) = (1, 1);
    for (i, raw) in src.split('\n').enumerate() {
        if i > 0 {
            // The line break sits just past the previous line's last char.
            out.push(Token::new(Tok::Newline, line, end, end + 1));
            line += 1;
        }
        let chars: Vec<char> = raw.strip_suffix('\r').unwrap_or(raw).chars().collect();
        scan_line(&chars, line, &mut out);
        end = chars.len() + 1;
    }
    out.push(Token::new(Tok::Eof, line, end, end));
    out
}

fn is_space(c: char) -> bool {
    c == ' ' || c == '\t'
}

/// Columns are 1-based: char index `i` is column `i + 1`.
fn tok(kind: Tok, line: usize, start: usize, end: usize) -> Token<Tok> {
    Token::new(kind, line, start + 1, end + 1)
}

fn text(chars: &[char], start: usize, end: usize) -> String {
    chars[start..end].iter().collect()
}

fn starts_comment(chars: &[char], i: usize) -> bool {
    chars.get(i) == Some(&'/') && chars.get(i + 1) == Some(&'/')
}

/// `[start, end)` with surrounding spaces and tabs removed.
fn trim(chars: &[char], mut start: usize, mut end: usize) -> (usize, usize) {
    while start < end && is_space(chars[start]) {
        start += 1;
    }
    while end > start && is_space(chars[end - 1]) {
        end -= 1;
    }
    (start, end)
}

fn scan_line(chars: &[char], line: usize, out: &mut Vec<Token<Tok>>) {
    let len = chars.len();
    let (first, last) = trim(chars, 0, len);
    if first == last {
        return;
    }
    if starts_comment(chars, first) {
        out.push(tok(Tok::Comment, line, first, last));
    } else if chars[first] == '[' {
        scan_section(chars, first, line, out);
    } else if let Some(eq) = chars.iter().position(|&c| c == '=') {
        // The key is taken exactly as written, spaces included: UMSWE
        // names custom-script entries with keys made of spaces only.
        if eq > 0 {
            out.push(tok(Tok::Key(text(chars, 0, eq)), line, 0, eq));
        }
        out.push(tok(Tok::Eq, line, eq, eq + 1));
        scan_values(chars, eq + 1, line, out);
    } else {
        out.push(tok(Tok::Junk(text(chars, first, last)), line, first, last));
    }
}

/// `[name]`, `[name` (unclosed), then an optional comment.
fn scan_section(chars: &[char], open: usize, line: usize, out: &mut Vec<Token<Tok>>) {
    out.push(tok(Tok::LBracket, line, open, open + 1));
    let name_start = open + 1;
    let name_end = chars[name_start..]
        .iter()
        .position(|&c| c == ']')
        .map_or(chars.len(), |p| name_start + p);
    let name = if name_end < chars.len() {
        (name_start, name_end)
    } else {
        // No `]`: the name runs to the end of the line, minus trailing space.
        (name_start, trim(chars, name_start, name_end).1.max(name_start))
    };
    if name.0 < name.1 {
        out.push(tok(Tok::SectionName(text(chars, name.0, name.1)), line, name.0, name.1));
    }
    if name_end == chars.len() {
        return;
    }
    out.push(tok(Tok::RBracket, line, name_end, name_end + 1));
    let (rest, last) = trim(chars, name_end + 1, chars.len());
    if rest == last {
        return;
    }
    let kind = if starts_comment(chars, rest) {
        Tok::Comment
    } else {
        Tok::Junk(text(chars, rest, last))
    };
    out.push(tok(kind, line, rest, last));
}

/// The value list after `=`: parts separated by commas, up to a comment.
fn scan_values(chars: &[char], mut i: usize, line: usize, out: &mut Vec<Token<Tok>>) {
    let len = chars.len();
    while i < len {
        let c = chars[i];
        if is_space(c) {
            i += 1;
        } else if c == ',' {
            out.push(tok(Tok::Comma, line, i, i + 1));
            i += 1;
        } else if starts_comment(chars, i) {
            let (_, last) = trim(chars, i, len);
            out.push(tok(Tok::Comment, line, i, last));
            return;
        } else if c == '"' {
            // No escapes: the next `"` closes the string. An unclosed one
            // runs to the end of the line.
            match chars[i + 1..].iter().position(|&c| c == '"') {
                Some(p) => {
                    let close = i + 1 + p;
                    let value = text(chars, i + 1, close);
                    out.push(tok(Tok::Quoted { value, closed: true }, line, i, close + 1));
                    i = close + 1;
                }
                None => {
                    let (_, last) = trim(chars, i, len);
                    let value = text(chars, i + 1, last);
                    out.push(tok(Tok::Quoted { value, closed: false }, line, i, last));
                    return;
                }
            }
        } else {
            let mut end = i;
            while end < len && chars[end] != ',' && chars[end] != '"' && !starts_comment(chars, end) {
                end += 1;
            }
            let (s, e) = trim(chars, i, end);
            out.push(tok(classify(&text(chars, s, e)), line, s, e));
            i = end;
        }
    }
}

/// An unquoted part is a number only if the whole part is one:
/// `-?digits` (an integer that fits `i64`) or `-?digits.digits` with
/// digits on at least one side of the point.
fn classify(part: &str) -> Tok {
    let digits = part.strip_prefix('-').unwrap_or(part);
    let all_digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    match digits.split_once('.') {
        None if !digits.is_empty() && all_digits(digits) => match part.parse() {
            Ok(n) => Tok::Int(n, part.to_string()),
            Err(_) => Tok::Text(part.to_string()),
        },
        Some((int, frac))
            if !(int.is_empty() && frac.is_empty()) && all_digits(int) && all_digits(frac) =>
        {
            Tok::Real(part.to_string())
        }
        _ => Tok::Text(part.to_string()),
    }
}

#[cfg(test)]
#[path = "lexer.test.rs"]
mod tests;
