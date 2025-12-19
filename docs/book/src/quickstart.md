# Quick Start

Add medley to your project:

```toml
[dependencies]
medley = "0.1"
```

Define a grammar with W3C EBNF syntax (`::=` for rules, `[]` for optional, `{}` for repetition):

```rust
use medley::ebnf::{grammar, ParseEvent, parse};
use std::io::Cursor;

let g = grammar! {
    record ::= field { ',' field };
    field  ::= word { word };
    word   ::= letter { letter };
    letter ::= 'a'..'z' | 'A'..'Z';
};

for ev in parse(&g, Cursor::new("alpha,beta".as_bytes())) {
    if let ParseEvent::Token { kind, .. } = ev {
        println!("token: {:?}", kind);
    }
}
```

Build a full AST when the input is small:

```rust
use medley::ebnf::grammar;
use medley::ast::parse_str;

let g = grammar! {
    expr  ::= term { op term };
    term  ::= digit { digit };
    digit ::= '0'..'9';
    op    ::= '+' | '-';
};

let ast = parse_str(&g, "12+34").expect("parse failed");
assert!(ast.metadata.success);
```
