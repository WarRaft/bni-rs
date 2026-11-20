use super::*;
use crate::parse;

#[test]
fn renders_sections_items_and_fields() {
    let tree = render_tree(&parse("top=x\n[S]\nk=1,,a\"b\"\n=\"open\n[T"));
    let expected = "\
Document
├─ Preamble
│  └─ Item top
│     └─ Text x
├─ Section [S]
│  ├─ Item k
│  │  ├─ Int 1
│  │  ├─ Empty
│  │  └─ Field
│  │     ├─ Text a
│  │     └─ Quoted \"b\"
│  └─ Item (no key)
│     └─ Quoted \"open (unterminated)
└─ Section [T (unclosed)
";
    assert_eq!(tree, expected);
}

#[test]
fn renders_dot() {
    assert!(render_dot(&parse("[S]\nk=1")).starts_with("digraph"));
}
