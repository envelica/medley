//! Abstract Syntax Tree (AST) for language parsing.
//!
//! Provides a complete, non-lazy AST builder that consumes all parse events
//! and constructs a full syntax tree. This is suitable for language parsing where the
//! entire source code is typically small enough to fit in memory.

mod node;
mod builder;

pub use node::{AstNode, Ast, AstMetadata};
pub use builder::AstBuilder;

/// Parse an input string using an EBNF grammar and build a complete AST.
///
/// This function consumes all parse events and constructs a full syntax tree.
/// Suitable for language parsing where the entire source code fits in memory.
///
/// # Arguments
/// * `grammar` - The EBNF grammar to parse with
/// * `input` - The input string to parse
///
/// # Returns
/// A complete AST or an error describing the parse failure
///
/// # Example
/// ```
/// use medley::ebnf::grammar;
/// use medley::ast;
///
/// let g = grammar! {
///     start = "hello";
/// };
/// let ast = ast::parse_str(&g, "hello").expect("parse failed");
/// ```
pub fn parse_str(grammar: &crate::ebnf::Grammar, input: &str) -> Result<Ast, String> {
    use crate::ebnf::{parse_str as ebnf_parse, ParseEvent, TokenKind};

    let mut builder = AstBuilder::new();

    for event in ebnf_parse(grammar, input) {
        match event {
            ParseEvent::Token { kind, span } => {
                let value = match kind {
                    TokenKind::Char(ch) => ch.to_string(),
                    TokenKind::Str(s) => s.to_string(),
                    TokenKind::Class(ch) => ch.to_string(),
                };
                builder.add_terminal(value, span);
            }
            ParseEvent::Error(err) => {
                return Err(format!(
                    "Parse error at position {}: {}",
                    err.position, err.message
                ));
            }
            _ => {} // Ignore Start/End events for now
        }
    }

    // Build the AST with collected tokens
    builder.build(input.len())
}
