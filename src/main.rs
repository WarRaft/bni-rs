use std::env;
use std::fs;
use std::process::ExitCode;

use bni_rs::{lint, render_dot, render_html, render_tree, Level};

enum Mode {
    Lint,
    Ast,
    Dot,
    Html,
}

fn usage() {
    eprintln!("usage: bni-rs [--ast|--dot|--html] <file.txt>");
    eprintln!("  (no flag)  lint the file and print diagnostics");
    eprintln!("  --ast      print the parsed document as an indented tree");
    eprintln!("  --dot      print the parsed document as a Graphviz DOT graph");
    eprintln!("  --html     print a self-contained AST viewer page (source + tree)");
    eprintln!();
    eprintln!("  e.g. bni-rs --html file.txt > file.ast.html && open file.ast.html");
}

fn main() -> ExitCode {
    let mut mode = Mode::Lint;
    let mut path = None;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--ast" => mode = Mode::Ast,
            "--dot" => mode = Mode::Dot,
            "--html" => mode = Mode::Html,
            "-h" | "--help" => {
                usage();
                return ExitCode::SUCCESS;
            }
            other => path = Some(other.to_string()),
        }
    }

    let Some(path) = path else {
        usage();
        return ExitCode::FAILURE;
    };

    let src = match fs::read_to_string(&path) {
        Ok(src) => src,
        Err(err) => {
            eprintln!("{path}: {err}");
            return ExitCode::FAILURE;
        }
    };

    // Parsing never fails as a whole; malformed lines are errors.
    let doc = bni_rs::parse(&src);
    match mode {
        Mode::Ast => print!("{}", render_tree(&doc)),
        Mode::Dot => print!("{}", render_dot(&doc)),
        Mode::Html => print!("{}", render_html(&doc, &src, &path)),
        Mode::Lint => {
            let diagnostics = lint(&doc);
            for err in &doc.errors {
                println!("{path}:{}:{}: error: {}", err.line, err.col, err.message);
            }
            for diag in &diagnostics {
                println!("{path}:{}: {}: {}", diag.line, diag.level.as_str(), diag.message);
            }
            if doc.errors.is_empty() && diagnostics.is_empty() {
                println!("{path}: no issues found");
            }
            if !doc.errors.is_empty() || diagnostics.iter().any(|d| d.level == Level::Error) {
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
