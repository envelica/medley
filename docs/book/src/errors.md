# Errors & Diagnostics

`ParseEvent::Error` carries message, position, optional span, rule context, and hint.

```rust
use medley::ebnf::{grammar, parse_str, ParseEvent};

let g = grammar! { start ::= "hello"; };
let events: Vec<_> = parse_str(&g, "bye").collect();

for ev in events {
    if let ParseEvent::Error(err) = ev {
        eprintln!("rule: {:?}, pos: {}, msg: {}", err.rule_context, err.position, err.message);
    }
}
```

Spans track line/column for string inputs. For streaming readers, spans are relative to consumed data; combine with your own offsets when chunking.
