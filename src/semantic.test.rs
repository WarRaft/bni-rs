use super::*;

fn at(src: &str, line: usize, col: usize) -> Option<(T, Modifiers, usize)> {
    semantic_tokens(src)
        .into_iter()
        .find(|t| t.line == line && t.start_col == col)
        .map(|t| (t.ty, t.mods, t.end_col))
}

#[test]
fn classifies_line_roles() {
    let src = "// c\n[Sec]\nKey=1,0.5,\"q\",text // tail\nF=~Player,WESTRING_X,TRIGSTR_001";
    let decl = Modifiers::of(&[M::Declaration]);
    assert_eq!(at(src, 1, 1), Some((T::Comment, Modifiers::NONE, 5)));
    assert_eq!(at(src, 2, 2), Some((T::Namespace, decl, 5)));
    assert_eq!(at(src, 3, 1), Some((T::Property, decl, 4)));
    assert_eq!(at(src, 3, 4), Some((T::Operator, Modifiers::NONE, 5)));
    assert_eq!(at(src, 3, 5).map(|t| t.0), Some(T::Number));
    assert_eq!(at(src, 3, 7).map(|t| t.0), Some(T::Number));
    assert_eq!(at(src, 3, 11), Some((T::String, Modifiers::NONE, 14)));
    assert_eq!(at(src, 3, 15), Some((T::String, Modifiers::NONE, 19)));
    assert_eq!(at(src, 3, 20).map(|t| t.0), Some(T::Comment));
    assert_eq!(at(src, 4, 3).map(|t| t.0), Some(T::Parameter));
    let readonly = Modifiers::of(&[M::Readonly]);
    assert_eq!(at(src, 4, 11), Some((T::Variable, readonly, 21)));
    assert_eq!(at(src, 4, 22), Some((T::Variable, readonly, 33)));
}

#[test]
fn brackets_commas_and_junk_are_not_tokens() {
    let toks = semantic_tokens("[A]\nk=a,b\njunk");
    assert_eq!(toks.len(), 5); // A, k, =, a, b
}

#[test]
fn encodes_for_the_editor() {
    // `[S]`: one token at char 1, length 1, namespace + declaration.
    let data = encoded_semantic_tokens("[S]");
    assert_eq!(data, [0, 1, 1, T::Namespace.index(), M::Declaration.bit()]);
}
