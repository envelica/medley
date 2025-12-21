# Grammar Syntax (W3C EBNF)

Use W3C-style EBNF in `grammar!`:

```rust
grammar! {
    // Rule definitions
    sentence ::= subject verb object;

    // Alternation
    subject ::= "alice" | "bob";

    // Optional
    object ::= noun [ adjective ];

    // Repetition (zero or more)
    noun ::= letter { letter };

    // Grouping
    verb ::= ("eats" | "reads");

    // Ranges
    letter ::= 'a'..'z' | 'A'..'Z';
}
```

Notes:
- Rules must use `::=`.
- `[]` is optional, `{}` is zero-or-more; one-or-more is `item { item }`.
- Ranges use `'a'..'z'` (inclusive).
- Terminals can be `'c'` or `"str"`.
- Numeric character references are supported using `#x..` (hex) or `#..` (decimal), e.g. `#x9` for TAB or `#10` for newline.
