//! Semantic highlighting for BNI, straight from the scanner's tokens (the
//! same tokens the parser reads), so it works on any file, broken or not:
//!
//! - section names: `namespace` + `declaration`;
//! - keys: `property` + `declaration`; `=`: `operator`;
//! - numbers: `number`; quoted and unquoted text: `string`;
//! - `~Name` parts (parameters in `TriggerStrings.txt`): `parameter`;
//! - `WESTRING_*` / `TRIGSTR_*` parts (string-table references):
//!   `variable` + `readonly`;
//! - comments: `comment`.

use crate::lexer::tokenize;
use crate::token::Tok;
use parser_rs::semantic;
use parser_rs::{Modifiers, SemanticModifier as M, SemanticToken, SemanticType as T};

/// Every semantic token in `src`, unencoded.
pub fn semantic_tokens(src: &str) -> Vec<SemanticToken> {
    let decl = Modifiers::of(&[M::Declaration]);
    tokenize(src)
        .iter()
        .filter_map(|t| {
            let (ty, mods) = match &t.kind {
                Tok::SectionName(_) => (T::Namespace, decl),
                Tok::Key(_) => (T::Property, decl),
                Tok::Eq => (T::Operator, Modifiers::NONE),
                Tok::Int(_) | Tok::Real(_) => (T::Number, Modifiers::NONE),
                Tok::Quoted { .. } => (T::String, Modifiers::NONE),
                Tok::Text(s) => text_type(s),
                Tok::Comment => (T::Comment, Modifiers::NONE),
                _ => return None,
            };
            Some(SemanticToken::at(t, ty, mods))
        })
        .collect()
}

/// [`semantic_tokens`] in the editor's wire format.
pub fn encoded_semantic_tokens(src: &str) -> Vec<u32> {
    semantic::encode(semantic_tokens(src), src)
}

fn text_type(s: &str) -> (T, Modifiers) {
    let ident = |rest: &str| !rest.is_empty() && rest.chars().all(|c| c.is_alphanumeric() || c == '_');
    if s.strip_prefix('~').is_some_and(|rest| !rest.is_empty()) {
        (T::Parameter, Modifiers::NONE)
    } else if ["WESTRING_", "TRIGSTR_"]
        .iter()
        .any(|p| s.strip_prefix(p).is_some_and(ident))
    {
        (T::Variable, Modifiers::of(&[M::Readonly]))
    } else {
        (T::String, Modifiers::NONE)
    }
}

#[cfg(test)]
#[path = "semantic.test.rs"]
mod tests;
