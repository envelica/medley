//! EBNF module root using newer Rust module style (file + subfolder).
//!
//! Submodules live under `src/ebnf/` (e.g., `ir.rs`).
//! This file declares and re-exports them for public use.

mod ir;

pub use ir::*;
