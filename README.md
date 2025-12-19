# medley

**A collection of high-quality, generic Rust utility modules designed for maximum convenience and minimal dependency footprint.**

[![crates.io](https://img.shields.io/crates/v/medley.svg)](https://crates.io/crates/medley)
[![License](https://img.shields.io/crates/l/medley.svg)](https://github.com/Envelica/medley/blob/main/LICENSE-MIT)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Envelica/medley/.github/workflows/rust.yml?branch=main)](https://github.com/Envelica/medley/actions)

## ⚠️ Stability Warning: Under Heavy Development

**The medley crate is currently under active, heavy development and is NOT ready for production use.**

We are working toward a stable **1.0.0** release, but until then:
* The API (**function names, module structure, and signatures**) may change drastically without prior warning.
* Functionality is actively being added, tested, and refactored.

We encourage review, testing, and feedback, but please do not rely on this crate in mission-critical applications until it reaches **1.0.0**.

## About

The purpose of medley is to aggregate various small, frequently needed utility features—from complex data structure extensions to simple trait implementations—into a single, well-organized crate. This allows developers to add one dependency (medley) instead of cluttering their Cargo.toml with multiple niche crates, thereby **keeping the overall dependency tree small and manageable.**

### Why medley?

* **Small Depencency Footprint:** Aggregates functionality with a strong focus on minimal dependencies.
* **Ergonomics:** Provides intuitive, easy-to-use APIs for common development needs.
* **Modular:** Modules are designed to be entirely independent, preventing feature bloat.

## Modules

Currently, medley provides:

### EBNF Parser (`medley::ebnf`)

A zero-copy, streaming EBNF parser with a compile-time `grammar!` macro. Perfect for parsing structured text with minimal memory overhead.

**Features:**
- Pull-based parsing (O(1) memory for large inputs)
- Optional AST building for small inputs
- Compile-time grammar validation
- Precise error reporting with spans

## Documentation & Links

- mdBook (GitHub Pages): https://envelica.github.io/medley/
- Examples: [examples/](examples/)
- Macro spec: [docs/grammar-macro-spec.md](docs/grammar-macro-spec.md)

### Grammar Syntax (W3C-style EBNF)

The `grammar!` macro uses W3C-style EBNF constructs with `::=`:

```rust
grammar! {
    // Sequences: match items in order
    greeting ::= "hello" " " name;

    // Alternation: match one of several options
    vowel    ::= 'a' | 'e' | 'i' | 'o' | 'u';

    // Optional: [ ... ]
    excited  ::= "hello" [ "!" ];

    // Repetition (zero or more): { ... }
    spaces   ::= { ' ' };

    // Ranges: 'a'..'z' (inclusive)
    letter   ::= 'a'..'z' | 'A'..'Z';
    digit    ::= '0'..'9';

    // One-or-more: combine item + repetition
    word     ::= letter { letter };

    // Grouping with parentheses
    sentence ::= word { ' ' word };
}
```
## Usage

Add medley to your `Cargo.toml`:

```toml
[dependencies]
medley = "0.1"  # Use the latest stable version when released
```

Core APIs:
- `grammar!` — declare W3C-style grammars with `::=`.
- `parse` / `parse_str` — stream parse events (`ParseEvent`) over any `BufRead` or `&str`.
- `ast::parse_str` — build a full AST when the input comfortably fits in memory.
- Errors surface as `ParseEvent::Error` with message, position, optional span, rule context, and hint.

### Quick Start: Streaming Parser

Process data streams with constant memory usage:

```rust
use medley::ebnf::{grammar, parse, ParseEvent};
use std::io::Cursor;

let grammar = grammar! {
    record ::= field { ',' field };
    letter ::= 'a'..'z';
    field  ::= letter { letter };
};

for event in parse(&grammar, Cursor::new(b"alpha,beta,gamma")) {
    match event {
        ParseEvent::Token { kind, .. } => {
            // Process each token as it's parsed
        }
        ParseEvent::Error(e) => eprintln!("Parse error: {}", e.message),
        _ => {}
    }
}
```

Errors are delivered as `ParseEvent::Error` with message, position, optional span, rule context, and hint. Inspect them instead of panicking.

### Quick Start: AST Building

Build complete syntax trees for small inputs:

```rust
use medley::ebnf::grammar;
use medley::ast::parse_str;

let grammar = grammar! {
    expr  ::= term { op term };
    term  ::= digit { digit };
    digit ::= '0'..'9';
    op    ::= '+' | '-';
};

let ast = parse_str(&grammar, "12+34").expect("parse failed");
println!("Parsed {} terminals", ast.collect_terminals().len());
```

### Quick Start: AST Visitor Pattern

Traverse and transform syntax trees:

```rust
use medley::ast::{Visitor, parse_str};
use medley::ebnf::{grammar, Span};

struct TerminalCounter { count: usize }

impl Visitor for TerminalCounter {
    fn visit_terminal(&mut self, _value: &str, _span: &Span) {
        self.count += 1;
    }
}

let g = grammar! { start ::= "hello" " " "world"; };
let ast = parse_str(&g, "hello world").unwrap();

let mut counter = TerminalCounter { count: 0 };
counter.visit_ast(&ast);
assert_eq!(counter.count, 3);
```

## Examples

Explore complete, runnable examples in the [`examples/`](examples/) directory:

### Expression Evaluator
Pull-parse and evaluate arithmetic expressions:

```bash
cargo run --example expr_pull
```

```rust
// Parses "12+30" and computes the result
let grammar = grammar! {
    expr  ::= num op num;
    num   ::= digit { digit };
    digit ::= '0'..'9';
    op    ::= '+' | '-';
};
// Output: 12+30 = 42
```

### CSV Stream Parser
Process CSV data with constant memory:

```bash
cargo run --example csv_pull
```

```rust
// Parses CSV records field-by-field
let grammar = grammar! {
    record ::= field { ',' field };
    field  ::= word { word };
    word   ::= 'a'..'z' | 'A'..'Z' | '0'..'9';
};
// Output: record: ["alpha", "beta", "gamma"]
```

### AST Visitor
Custom tree traversal and analysis:

```bash
cargo run --example ast_visitor
```

Demonstrates counting terminals, collecting rule names, and pretty-printing AST structure.

### AST Transformer
In-place tree transformations:

```bash
cargo run --example ast_transform
```

Shows how to use `VisitorMut` to transform terminal values to uppercase.

### Performance Benchmarks

Measure parsing performance:

```bash
# Small input benchmark (100K iterations)
cargo run --example parse_small

# Large stream benchmark (1MB, chunked reading)
cargo run --example parse_stream --release
```

### W3C EBNF Grammar (Example)

The `examples/w3c_ebnf.rs` file contains a grammar for a common EBNF dialect
expressible with `grammar!`. It demonstrates sequences, alternations, grouping,
optional/repetition (`[]` / `{}`), terminals, and ranges. Run it with:

```bash
cargo run --example w3c_ebnf
```

## Troubleshooting & Limitations

- Rules must use `::=`; using `=` will fail during macro expansion.
- `[]` (optional) and `{}` (zero-or-more) replace postfix `? * +`; one-or-more is `item { item }`.
- Ranges use `'a'..'z'` instead of `[a-z]`.
- Streaming backtracking is bounded to the current buffer window; extremely deep backtracking over huge inputs may fail.
- Spans are best-effort when streaming; for precise spans, prefer `parse_str` on in-memory text.