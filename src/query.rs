//! Position-based lookup over a parsed [`Document`]: "which node is at
//! (or covers) this location". Coordinates are [`Span`]'s (line,
//! codepoint column).

use crate::ast::*;

/// A reference to whichever kind of node a query matched.
#[derive(Debug, Clone, Copy)]
pub enum AstRef<'a> {
    Section(&'a Section),
    Item(&'a Item),
    Field(&'a Field),
    Value(&'a Value),
}

impl AstRef<'_> {
    /// Sections and items cover their whole line; fields and values an
    /// exact span.
    pub fn span(&self) -> Span {
        match self {
            AstRef::Section(s) => Span::whole_line(s.line),
            AstRef::Item(i) => Span::whole_line(i.line),
            AstRef::Field(f) => f.span,
            AstRef::Value(v) => v.span,
        }
    }
}

/// Finds the most specific node whose span contains the single-line range
/// `[start_col, end_col)` on `line`.
pub fn find_node_in(doc: &Document, line: usize, start_col: usize, end_col: usize) -> Option<AstRef<'_>> {
    let q = (line, start_col, end_col);
    if let Some(found) = doc.preamble.iter().find_map(|i| in_item(i, q)) {
        return Some(found);
    }
    for section in &doc.sections {
        if section.line == line {
            return Some(AstRef::Section(section));
        }
        if let Some(found) = section.items.iter().find_map(|i| in_item(i, q)) {
            return Some(found);
        }
    }
    None
}

fn in_item(item: &Item, (line, sc, ec): (usize, usize, usize)) -> Option<AstRef<'_>> {
    if item.line != line {
        return None;
    }
    let hit = |node: AstRef<'_>| node.span().contains(line, sc, ec);
    for field in &item.fields {
        if let Some(v) = field.parts.iter().find(|v| hit(AstRef::Value(v))) {
            return Some(AstRef::Value(v));
        }
        if field.parts.len() > 1 && hit(AstRef::Field(field)) {
            return Some(AstRef::Field(field));
        }
    }
    Some(AstRef::Item(item))
}

/// Finds the most specific node at a single position.
pub fn find_node_at(doc: &Document, line: usize, col: usize) -> Option<AstRef<'_>> {
    find_node_in(doc, line, col, col)
}

#[cfg(test)]
#[path = "query.test.rs"]
mod tests;
