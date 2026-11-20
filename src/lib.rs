//! A parser, linter and semantic highlighter for BNI (Blizzard INI), the
//! data-file format of Warcraft III's `TriggerData.txt`,
//! `TriggerStrings.txt`, `WorldEditStrings.txt` and friends. Structured
//! like the other language crates; everything not BNI-specific comes from
//! `parser-rs` (see its README for the shared design).

pub mod ast;
pub mod html;
pub mod lexer;
pub mod lint;
pub mod parser;
pub mod printer;
pub mod query;
pub mod semantic;
pub mod token;

pub use ast::{ColumnEncoding, Document, Span};
pub use html::render_html;
pub use lint::{lint, Diagnostic, Level};
pub use parser::{ParseError, Parser};
pub use printer::{render_dot, render_tree};
pub use query::{find_node_at, find_node_in, AstRef};
pub use semantic::{encoded_semantic_tokens, semantic_tokens};

/// Parses BNI source. This never fails as a whole: malformed lines are
/// skipped and listed in [`Document::errors`].
pub fn parse(src: &str) -> Document {
    Parser::parse_str(src)
}
