use crate::ast::*;
pub use parser_rs::Node;

/// Builds a visualizable tree out of a parsed document.
pub fn build_tree(doc: &Document) -> Node {
    let mut children = Vec::new();
    if !doc.preamble.is_empty() {
        children.push(Node::with_children("Preamble", items(&doc.preamble)).tag("container", None));
    }
    children.extend(doc.sections.iter().map(section_node));
    Node::with_children("Document", children).tag("root", None)
}

/// Renders the document as an indented tree.
pub fn render_tree(doc: &Document) -> String {
    parser_rs::render_tree(&build_tree(doc))
}

/// Renders the document as a Graphviz DOT graph.
pub fn render_dot(doc: &Document) -> String {
    parser_rs::render_dot(&build_tree(doc))
}

fn section_node(s: &Section) -> Node {
    let label = if s.closed {
        format!("Section [{}]", s.name.text)
    } else {
        format!("Section [{} (unclosed)", s.name.text)
    };
    Node::with_children(label, items(&s.items)).tag("decl", Some(Span::whole_line(s.line)))
}

fn items(items: &[Item]) -> Vec<Node> {
    items.iter().map(item_node).collect()
}

fn item_node(item: &Item) -> Node {
    let label = match &item.key {
        Some(k) => format!("Item {}", k.text),
        None => "Item (no key)".to_string(),
    };
    let fields = item.fields.iter().map(field_node).collect();
    Node::with_children(label, fields).tag("stmt", Some(Span::whole_line(item.line)))
}

/// A one-part field is shown as that part; others as a `Field` node.
fn field_node(field: &Field) -> Node {
    match field.parts.as_slice() {
        [] => Node::leaf("Empty").tag("lit", Some(field.span)),
        [part] => value_node(part),
        parts => Node::with_children("Field", parts.iter().map(value_node).collect())
            .tag("expr", Some(field.span)),
    }
}

fn value_node(v: &Value) -> Node {
    let label = match &v.kind {
        ValueKind::Int(n) => format!("Int {n}"),
        ValueKind::Real(_) => format!("Real {}", v.text),
        ValueKind::Text => format!("Text {}", v.text),
        ValueKind::Quoted { closed: true } => format!("Quoted \"{}\"", v.text),
        ValueKind::Quoted { closed: false } => format!("Quoted \"{} (unterminated)", v.text),
    };
    Node::leaf(label).tag("lit", Some(v.span))
}

#[cfg(test)]
#[path = "printer.test.rs"]
mod tests;
