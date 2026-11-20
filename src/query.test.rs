use super::*;
use crate::parse;

#[test]
fn finds_the_deepest_node() {
    let doc = parse("[S]\nkey=1,a\"b\"\nother=2");
    assert!(matches!(find_node_at(&doc, 1, 2), Some(AstRef::Section(s)) if s.name.text == "S"));
    assert!(matches!(find_node_at(&doc, 2, 1), Some(AstRef::Item(i)) if i.line == 2));
    assert!(matches!(find_node_at(&doc, 2, 5), Some(AstRef::Value(v)) if v.kind == ValueKind::Int(1)));
    assert!(matches!(find_node_at(&doc, 2, 9), Some(AstRef::Value(v)) if v.text == "b"));
    assert!(matches!(find_node_in(&doc, 2, 7, 10), Some(AstRef::Field(f)) if f.text() == "ab"));
    assert!(matches!(find_node_at(&doc, 3, 1), Some(AstRef::Item(_))));
    assert!(find_node_at(&doc, 9, 1).is_none());
}
