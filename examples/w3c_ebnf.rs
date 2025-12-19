// Example: A minimal EBNF-style grammar (W3C-flavored) expressed with grammar!
// This does not cover every nuance of the W3C spec, but demonstrates
// that common EBNF constructs are fully supported by medley.

use medley::ebnf::{ParseEvent, TokenKind, grammar, parse};
use std::io::Cursor;

fn main() {
    // Grammar for a simple W3C-style EBNF dialect:
    // rule     ::= ident '=' expr ';'
    // expr     = term { '|' term }
    // term     = factor { factor }
    // factor   = primary [ op ]
    // op       = '+' | '*' | '?'
    // primary  = ident | string | range | '(' expr ')'
    // ident    = letter { letter | digit | '_' }
    // string   = '"' { any-but-quote } '"'
    // range    = '\'' char '\'' '..' '\'' char '\''
    // letter   = 'a'..'z' | 'A'..'Z'
    // digit    = '0'..'9'

    let g = grammar! {
        rule    ::= ident "=" expr ";";
        expr    ::= term { '|' term };
        term    ::= factor { factor };
        factor  ::= primary [ op ];
        op      ::= '+' | '*' | '?';
        primary ::= ident | string | range | '(' expr ')';

        ident   ::= letter { (letter | digit | '_') };
        string  ::= '"' { string_char } '"';
        string_char ::= 'a'..'z' | 'A'..'Z' | '0'..'9' | '_' | ' ';

        range   ::= 'a'..'z' | 'A'..'Z' | '0'..'9'; // Demonstration of range tokens

        letter  ::= 'a'..'z' | 'A'..'Z';
        digit   ::= '0'..'9';
    };

    let input = "digit = '0'..'9' { '0'..'9' };\nident = ('a'..'z'|'A'..'Z'|'_') { ('a'..'z'|'A'..'Z'|'_'|'0'..'9') };";

    println!("EBNF input:\n{}\n\nTokens:", input);
    for ev in parse(&g, Cursor::new(input.as_bytes())) {
        if let ParseEvent::Token { kind, .. } = ev {
            match kind {
                TokenKind::Char(c) | TokenKind::Class(c) => print!("{}", c),
                TokenKind::Str(s) => print!("{}", s),
            }
        }
    }
    println!();
}
