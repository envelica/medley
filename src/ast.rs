//! Abstract Syntax Tree (AST) for language parsing.
//!
//! Provides a complete, non-lazy AST builder that consumes all parse events
//! and constructs a full syntax tree. This is suitable for language parsing where the
//! entire source code is typically small enough to fit in memory.
//!
//! # Memory Tradeoffs
//!
//! The AST module offers an **on-demand** approach to building complete syntax trees.
//! Understanding the tradeoffs is important for choosing the right parsing strategy:
//!
//! ## Pull Parsing (Zero-Copy, Streaming)
//!
//! **When to use:** Processing large files, streaming data, or memory-constrained environments.
//!
//! - **Memory:** O(1) - Constant memory usage, only buffering small chunks
//! - **Performance:** Fastest for large inputs, no allocation overhead
//! - **Flexibility:** Can process infinite streams, stop early, skip irrelevant data
//! - **Use case:** Log processing, CSV parsing, data validation, large document scanning
//!
//! ```ignore
//! use medley::ebnf::{grammar, parse, ParseEvent};
//! use std::io::Cursor;
//!
//! let g = grammar! { record ::= letter { letter } { ',' letter { letter } }; letter ::= 'a'..'z'; };
//! for event in parse(&g, Cursor::new(b"a,b,c")) {
//!     match event {
//!         ParseEvent::Token { .. } => { /* process */ }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ## AST Building (Full Tree)
//!
//! **When to use:** Need random access, multiple passes, or tree transformations.
//!
//! - **Memory:** O(n) - Proportional to input size, stores entire tree
//! - **Performance:** Additional allocation cost, slower for very large inputs
//! - **Flexibility:** Easy random access, tree transformations, multiple passes
//! - **Use case:** Programming language parsing, JSON/XML processing, syntax analysis
//!
//! ```
//! use medley::ebnf::grammar;
//! use medley::ast::parse_str;
//!
//! let g = grammar! { start ::= "hello"; };
//! let ast = parse_str(&g, "hello").expect("parse failed");
//!
//! // Can now traverse the tree multiple times
//! let terminals = ast.collect_terminals();
//! let depth = ast.depth();
//! ```
//!
//! ## Visitor Pattern (AST Traversal)
//!
//! For operations on existing ASTs, use the visitor pattern to avoid modifying the core structure:
//!
//! ```
//! use medley::ast::{Visitor, parse_str};
//! use medley::ebnf::{grammar, Span};
//!
//! struct TerminalCounter { count: usize }
//!
//! impl Visitor for TerminalCounter {
//!     fn visit_terminal(&mut self, _value: &str, _span: &Span) {
//!         self.count += 1;
//!     }
//! }
//!
//! let g = grammar! { start ::= "a" "b"; };
//! let ast = parse_str(&g, "ab").unwrap();
//! let mut counter = TerminalCounter { count: 0 };
//! counter.visit_ast(&ast);
//! assert_eq!(counter.count, 2);
//! ```
//!
//! ## Decision Guide
//!
//! Choose **Pull Parsing** when:
//! - Input size > 10MB
//! - Memory is constrained
//! - Only need one pass through data
//! - Processing can stop early
//!
//! Choose **AST Building** when:
//! - Input size < 1MB (typical source files)
//! - Need multiple passes or transformations
//! - Random access to tree nodes required
//! - Building IDE features or static analysis tools

mod builder;
mod node;
mod visitor;

pub use builder::AstBuilder;
pub use node::{Ast, AstMetadata, AstNode};
pub use visitor::{Visitor, VisitorMut};

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
///     start ::= "hello";
/// };
/// let ast = ast::parse_str(&g, "hello").expect("parse failed");
/// ```
pub fn parse_str(grammar: &crate::ebnf::Grammar, input: &str) -> Result<Ast, String> {
    use crate::ebnf::{ParseEvent, TokenKind, parse_str as ebnf_parse};

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
