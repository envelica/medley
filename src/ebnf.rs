//! EBNF module root using newer Rust module style (file + subfolder).
//!
//! Submodules live under `src/ebnf/` (e.g., `ir.rs`).
//! This file declares and re-exports them for public use.

mod ir;
mod parser;

pub use ir::*;
pub use parser::*;

// Re-export the grammar! macro from medley-macros
pub use medley_macros::grammar;
