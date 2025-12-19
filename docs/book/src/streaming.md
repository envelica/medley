# Streaming Parser

`parse` and `parse_str` return iterators of `ParseEvent`, letting you stream input with O(1) memory.

```rust
use medley::ebnf::{grammar, parse, ParseEvent, TokenKind};
use std::io::Cursor;

let g = grammar! {
    record ::= field { ',' field };
    field  ::= word { word };
    word   ::= letter { letter };
    letter ::= 'a'..'z' | 'A'..'Z';
};

let input = Cursor::new(b"alpha,beta");
for ev in parse(&g, input) {
    match ev {
        ParseEvent::Token { kind: TokenKind::Str(s), span } => {
            println!("token {s} @ {:?}", span);
        }
        ParseEvent::Error(err) => eprintln!("error: {}", err.message),
        _ => {}
    }
}
```

Key points:
- Backtracking is bounded to the current buffer window (tuned for streaming).
- Use `ParseEvent::Error` to surface failures without panicking.
- For byte slices, `parse_str` is a shorthand: `parse_str(&g, "text")`.
