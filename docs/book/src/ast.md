# AST Building

For small inputs, build a full AST with `medley::ast::parse_str`.

```rust
use medley::ebnf::grammar;
use medley::ast::{parse_str, Visitor};
use medley::ebnf::Span;

let g = grammar! {
    expr  ::= term { op term };
    term  ::= digit { digit };
    digit ::= '0'..'9';
    op    ::= '+' | '-';
};

let mut ast = parse_str(&g, "12+34").expect("parse failed");

struct Counter { n: usize }
impl Visitor for Counter {
    fn visit_terminal(&mut self, _value: &str, _span: &Span) {
        self.n += 1;
    }
}

let mut counter = Counter { n: 0 };
counter.visit_ast(&ast);
assert_eq!(counter.n, 4);
```

Use `VisitorMut` to transform nodes in-place (see `examples/ast_transform.rs`).
