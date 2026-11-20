//! Lint rules for BNI. Only things that are almost certainly mistakes are
//! reported. Repeated keys are **not**: `TriggerStrings.txt` repeats a key
//! on purpose (name, then format, then hint), and keyless `=value` lines
//! are a convention too. Malformed lines are parse errors, in
//! [`Document::errors`].

use std::collections::HashMap;

use crate::ast::*;

pub use parser_rs::{Diagnostic, Level};
use parser_rs::rule;

/// Runs every lint rule over a document and returns the warnings found,
/// sorted by source line.
pub fn lint(doc: &Document) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for item in &doc.preamble {
        let what = item.key.as_ref().map_or("item".to_string(), |k| format!("key '{}'", k.text));
        out.push(Diagnostic::warning(rule::ITEM_OUTSIDE_SECTION, format!("{what} is outside of any [section]"), item.line));
    }
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for section in &doc.sections {
        let name = section.name.text.as_str();
        if name.trim().is_empty() {
            out.push(Diagnostic::warning(rule::EMPTY_SECTION_NAME, "empty section name", section.line));
        } else if let Some(first) = seen.get(name) {
            out.push(Diagnostic::warning(rule::DUPLICATE_SECTION, 
                format!("duplicate section [{name}] (first declared at line {first})"),
                section.line,
            ));
        } else {
            seen.insert(name, section.line);
        }
        if !section.closed {
            out.push(Diagnostic::warning(rule::UNCLOSED_SECTION_HEADER, 
                format!("section header [{name} is missing its ']'"),
                section.line,
            ));
        }
    }
    for item in doc.preamble.iter().chain(doc.sections.iter().flat_map(|s| &s.items)) {
        for part in item.fields.iter().flat_map(|f| &f.parts) {
            if part.kind == (ValueKind::Quoted { closed: false }) {
                out.push(Diagnostic::warning(rule::UNTERMINATED_STRING, "unterminated string: missing closing '\"'", item.line));
            }
        }
    }
    out.sort_by_key(|d| d.line);
    out
}

#[cfg(test)]
#[path = "lint.test.rs"]
mod tests;
