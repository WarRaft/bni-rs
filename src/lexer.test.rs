use super::*;

/// `(kind, col, end_col)` of each token on `line`, newlines and EOF left out.
fn line_tokens(src: &str, line: usize) -> Vec<(Tok, usize, usize)> {
    tokenize(src)
        .into_iter()
        .filter(|t| t.line == line && !matches!(t.kind, Tok::Newline | Tok::Eof))
        .map(|t| (t.kind, t.col, t.end_col))
        .collect()
}

fn text(s: &str) -> Tok {
    Tok::Text(s.into())
}

#[test]
fn scans_section_headers() {
    assert_eq!(
        line_tokens("  [Camera] // c", 1),
        [
            (Tok::LBracket, 3, 4),
            (Tok::SectionName("Camera".into()), 4, 10),
            (Tok::RBracket, 10, 11),
            (Tok::Comment, 12, 16),
        ]
    );
    // No `]`: the name runs to the end of the line.
    assert_eq!(
        line_tokens("[Open  ", 1),
        [(Tok::LBracket, 1, 2), (Tok::SectionName("Open".into()), 2, 6)]
    );
    assert_eq!(line_tokens("[]", 1), [(Tok::LBracket, 1, 2), (Tok::RBracket, 2, 3)]);
    assert_eq!(line_tokens("[A] x", 1)[3], (Tok::Junk("x".into()), 5, 6));
}

#[test]
fn scans_keys_and_values() {
    assert_eq!(
        line_tokens("Color = 255, 0.5 ,x y // c", 1),
        [
            (Tok::Key("Color ".into()), 1, 7),
            (Tok::Eq, 7, 8),
            (Tok::Int(255, "255".into()), 9, 12),
            (Tok::Comma, 12, 13),
            (Tok::Real("0.5".into()), 14, 17),
            (Tok::Comma, 18, 19),
            (text("x y"), 19, 22),
            (Tok::Comment, 23, 27),
        ]
    );
    assert_eq!(line_tokens("=v", 1), [(Tok::Eq, 1, 2), (text("v"), 2, 3)]);
    assert_eq!(line_tokens("K=a=b", 1)[2], (text("a=b"), 3, 6));
    // Keys keep their spaces; one may even be nothing but spaces.
    assert_eq!(line_tokens("  =1", 1)[0], (Tok::Key("  ".into()), 1, 3));
    assert_eq!(line_tokens("_  _Category=x", 1)[0], (Tok::Key("_  _Category".into()), 1, 13));
}

#[test]
fn quoted_strings_have_no_escapes() {
    let q = |s: &str, closed| Tok::Quoted { value: s.into(), closed };
    assert_eq!(line_tokens(r#"K="\""#, 1)[2], (q("\\", true), 3, 6));
    assert_eq!(line_tokens(r#"K="a, // b""#, 1)[2], (q("a, // b", true), 3, 12));
    assert_eq!(line_tokens(r#"K=a"""#, 1)[2..], [(text("a"), 3, 4), (q("", true), 4, 6)]);
    // Unclosed: to the end of the line.
    assert_eq!(line_tokens(r#"K=~Code" "#, 1)[2..], [(text("~Code"), 3, 8), (q("", false), 8, 9)]);
    // Single quotes are ordinary characters.
    assert_eq!(line_tokens("K='Hide' it", 1)[2], (text("'Hide' it"), 3, 12));
}

#[test]
fn classifies_numbers_by_the_whole_part() {
    assert_eq!(classify("42"), Tok::Int(42, "42".into()));
    assert_eq!(classify("-1"), Tok::Int(-1, "-1".into()));
    // The raw text is kept: `-0` stays negative, `007` keeps its zeros.
    assert_eq!(classify("-0"), Tok::Int(0, "-0".into()));
    assert_eq!(classify("007"), Tok::Int(7, "007".into()));
    assert_eq!(classify("0.00"), Tok::Real("0.00".into()));
    assert_eq!(classify("-.5"), Tok::Real("-.5".into()));
    assert_eq!(classify("3."), Tok::Real("3.".into()));
    for s in ["-", ".", "1e5", "12abc", "1.2.3", "99999999999999999999"] {
        assert_eq!(classify(s), text(s), "{s}");
    }
}

#[test]
fn handles_line_endings_bom_and_junk() {
    let toks = tokenize("\u{FEFF}[A]\r\nk=1\r\nbroken line");
    let kinds: Vec<&Tok> = toks.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        [
            &Tok::LBracket,
            &Tok::SectionName("A".into()),
            &Tok::RBracket,
            &Tok::Newline,
            &Tok::Key("k".into()),
            &Tok::Eq,
            &Tok::Int(1, "1".into()),
            &Tok::Newline,
            &Tok::Junk("broken line".into()),
            &Tok::Eof,
        ]
    );
    assert_eq!((toks[3].line, toks[3].col), (1, 4));
    assert_eq!((toks[9].line, toks[9].col), (3, 12));
}
