use super::*;

#[test]
fn parses_sections_and_items() {
    let doc = Parser::parse_str("top=1\n[A]\nx=1,two\n\n// c\n[B\ny=");
    assert_eq!(doc.preamble.len(), 1);
    let names: Vec<_> = doc.sections.iter().map(|s| (s.name.text.as_str(), s.closed)).collect();
    assert_eq!(names, [("A", true), ("B", false)]);
    let x = &doc.sections[0].items[0];
    assert_eq!(x.key.as_ref().unwrap().text, "x");
    assert_eq!(x.value_text(), "1,two");
    assert_eq!(x.fields[0].parts[0].kind, ValueKind::Int(1));
    assert!(doc.sections[1].items[0].fields.is_empty());
    assert!(doc.errors.is_empty());
}

#[test]
fn keeps_empty_fields() {
    let doc = Parser::parse_str("[S]\nk=,a,,b,");
    let fields: Vec<String> = doc.sections[0].items[0].fields.iter().map(Field::text).collect();
    assert_eq!(fields, ["", "a", "", "b", ""]);
    let spans: Vec<(usize, usize)> = doc.sections[0].items[0]
        .fields
        .iter()
        .map(|f| (f.span.start_col, f.span.end_col))
        .collect();
    assert_eq!(spans, [(3, 3), (4, 5), (6, 6), (7, 8), (9, 9)]);
}

#[test]
fn joins_adjacent_parts_into_one_field() {
    let doc = Parser::parse_str("[S]\nk=a\"b, c\"d,e");
    let item = &doc.sections[0].items[0];
    assert_eq!(item.fields.len(), 2);
    assert_eq!(item.fields[0].parts.len(), 3);
    assert_eq!(item.fields[0].text(), "ab, cd");
    assert_eq!((item.fields[0].span.start_col, item.fields[0].span.end_col), (3, 11));
}

#[test]
fn malformed_lines_are_errors_and_parsing_goes_on() {
    let doc = Parser::parse_str("[S]\nno equals here\n[T] trailing\nk=v");
    let errors: Vec<_> = doc.errors.iter().map(|e| (e.line, e.col, e.end_col)).collect();
    assert_eq!(errors, [(2, 1, 15), (3, 5, 13)]);
    assert!(doc.errors[0].message.starts_with("expected '=' in 'no equals here'"));
    assert_eq!(doc.errors[1].message, "unexpected 'trailing' after the section header");
    assert_eq!(doc.sections[1].items[0].value_text(), "v");
}

#[test]
fn looks_up_sections_and_repeated_keys() {
    let doc = Parser::parse_str("[S]\nk=1\nk=\"two\"\n=3");
    let s = doc.section("S").unwrap();
    let values: Vec<String> = s.get("k").map(Item::value_text).collect();
    assert_eq!(values, ["1", "two"]);
    assert!(s.items[2].key.is_none());
    assert!(doc.section("s").is_none());
}
