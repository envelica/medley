//! EBNF grammar definition and parsing.
//!
//! This module provides EBNF grammar types and a streaming parser
//! for parsing input according to EBNF grammars.

mod grammar;
mod parser;
mod span;

pub use grammar::*;
pub use parser::*;
pub use span::*;

// Re-export the grammar! macro
pub use medley_macros::grammar;
