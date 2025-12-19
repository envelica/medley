# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Work in progress toward initial release.

### Added

- **W3C-style EBNF grammar macro** (`grammar!`)
  - Compile-time grammar validation and code generation
  - `::=` rule definitions
  - Optional `[ ... ]` and repetition `{ ... }` constructs
  - Character ranges `'a'..'z'`
  - Alternation, grouping, sequences
  
- **Streaming pull parser** (`parse`, `parse_str`)
  - Zero-copy, O(1) memory for large inputs
  - Iterator-based `ParseEvent` API
  - Bounded backtracking with sliding window buffer
  - UTF-8 support with line/column tracking
  
- **Optional AST building** (`ast::parse_str`)
  - Full syntax tree construction for small inputs
  - Visitor and VisitorMut patterns for traversal and transformation
  - Metadata tracking (token count, input length, success status)
  
- **Comprehensive error reporting**
  - `ParseError` with message, position, span, rule context, and hints
  - Precise diagnostics for grammar validation and parsing failures
  
- **Examples**
  - Expression evaluator (pull parser)
  - CSV stream parser
  - AST visitor and transformer
  - Performance benchmarks
  - W3C EBNF demonstration
  
- **Documentation**
  - mdBook published to GitHub Pages
  - README with quickstart, examples, and troubleshooting
  - Inline documentation and doctests
  - Grammar macro specification

### Notes

- Minimum Rust version: 1.85
- Dual licensed under MIT OR Apache-2.0
- Zero runtime dependencies (proc-macro dependencies are build-time only)
