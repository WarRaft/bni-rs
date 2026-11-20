use bni_rs::ast::ValueKind;
use bni_rs::{lint, parse};
use std::path::Path;

const SAMPLE: &str = include_str!("fixtures/sample.txt");

#[test]
fn sample_file_parses_cleanly() {
    let doc = parse(SAMPLE);
    assert!(doc.errors.is_empty(), "{:?}", doc.errors);
    let sections: Vec<&str> = doc.sections.iter().map(|s| s.name.text.as_str()).collect();
    assert_eq!(sections, ["TriggerCategories", "TriggerTypes", "Misc", "TriggerActionStrings"]);
    let lint = lint(&doc);
    assert!(lint.is_empty(), "expected sample.txt to be clean, got {lint:?}");
}

#[test]
fn reads_typed_values() {
    let doc = parse(SAMPLE);
    let misc = doc.section("Misc").unwrap();
    let kinds = |key| -> Vec<ValueKind> {
        misc.get(key).next().unwrap().fields.iter().map(|f| f.parts[0].kind.clone()).collect()
    };
    assert_eq!(kinds("GoldTextVelocity"), [ValueKind::Int(0), ValueKind::Real(0.03), ValueKind::Int(100)]);
    assert_eq!(kinds("PuffDuration"), [ValueKind::Int(3000)]);
    assert_eq!(misc.get("Name").next().unwrap().value_text(), "City Building (Diagonal 1, Red)");
    assert_eq!(misc.get("WESTRING_HOTKEY_BACKSLASH").next().unwrap().value_text(), "\\");
    assert_eq!(misc.get("QuickSave").next().unwrap().value_text(), "'S'");
}

/// The real UMSWE / UJAPI data files from the old tree-sitter grammar's
/// repo, when it's checked out next to this one: they must parse without
/// a single malformed line.
#[test]
fn real_world_files_parse_without_errors() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tree-sitter-bni/example");
    let Ok(editors) = std::fs::read_dir(&dir) else {
        return;
    };
    for editor in editors.flatten() {
        for file in std::fs::read_dir(editor.path().join("ui")).into_iter().flatten().flatten() {
            let src = std::fs::read_to_string(file.path()).unwrap();
            let doc = parse(&src);
            assert!(doc.errors.is_empty(), "{}: {:?}", file.path().display(), &doc.errors[..1]);
            assert!(!doc.sections.is_empty());
        }
    }
}
