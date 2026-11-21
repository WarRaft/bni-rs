use super::*;

fn back(value: &str) -> Result<String, ParseError> {
    let doc = crate::parse(&format!("[s]\nk={value}\n"));
    assert_eq!(doc.errors, vec![], "{value}");
    unquote(&doc.sections[0].items[0].fields[0])
}

#[test]
fn quoted_strings_read_back() {
    for s in ["", "plain", "a, b // c", "Line one\nLine two\r\n", "say \"hi\"\t\u{1}", "\n", "ü 日本"] {
        let text = quote(s);
        assert_eq!(back(&text).unwrap(), s, "{text}");
    }
    assert_eq!(quote("a\nb"), "\"a\"\\n\"b\"");
}

#[test]
fn bad_escapes() {
    assert!(back("\"a\"\\x\"b\"").unwrap_err().message.contains("only escapes"));
    assert!(back("\"a\"\\u{0}").unwrap_err().message.contains("not 0"));
    assert!(back("\"open").unwrap_err().message.contains("isn't closed"));
}
