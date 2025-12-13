//! EBNF module: embeddable grammar definitions for parsing.
//!
//! This module provides a `grammar!` macro for defining EBNF grammars in token-tree form,
//! and supporting types for grammar representation, parsing, and diagnostics.
//!
//! # Overview
//!
//! The EBNF module is designed to be the foundation for building custom parsers in the `medley` crate.
//! It follows a phased implementation:
//!
//! - **Phase 1**: Specification and module structure (current).
//! - **Phase 2**: Grammar IR (internal representation) types.
//! - **Phase 3**: `grammar!` macro implementation.
//! - **Phase 4+**: Runtime parser, diagnostics, examples, tests.
//!
//! See [docs/grammar-macro-spec.md](../docs/grammar-macro-spec.md) for the full specification.
