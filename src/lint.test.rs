use super::*;
use crate::parse;

fn messages(src: &str) -> Vec<(usize, String)> {
    lint(&parse(src)).into_iter().map(|d| (d.line, d.message)).collect()
}

#[test]
fn reports_section_problems() {
    let m = messages("[A]\n[B\n[]\n[A]");
    assert_eq!(
        m,
        [
            (2, "section header [B is missing its ']'".to_string()),
            (3, "empty section name".to_string()),
            (4, "duplicate section [A] (first declared at line 1)".to_string()),
        ]
    );
}

#[test]
fn reports_items_outside_sections_and_unterminated_strings() {
    let m = messages("k=1\n=2\n[S]\nx=\"open");
    assert_eq!(
        m,
        [
            (1, "key 'k' is outside of any [section]".to_string()),
            (2, "item is outside of any [section]".to_string()),
            (4, "unterminated string: missing closing '\"'".to_string()),
        ]
    );
}

#[test]
fn repeated_keys_and_keyless_items_are_fine() {
    assert!(messages("[S]\nk=1\nk=2\n  =3\n=4").is_empty());
}
