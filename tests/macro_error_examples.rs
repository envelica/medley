// Examples demonstrating improved macro error messages with spans
// Uncomment individual tests to see the improved error messages

use medley::ebnf::grammar;

// Example 1: Missing semicolon
// #[test]
// fn missing_semicolon() {
//     let g = grammar! {
//         expr ::= term
//         term ::= digit
//     };
// }

// Example 2: Unexpected punctuation
// #[test]
// fn unexpected_punctuation() {
//     let g = grammar! {
//         expr ::= @ term;
//     };
// }

// Example 3: Invalid character literal
// #[test]
// fn invalid_char_literal() {
//     let g = grammar! {
//         expr ::= 'ab';  // Too many characters
//     };
// }

// Example 4: Empty grammar
// #[test]
// fn empty_grammar() {
//     let g = grammar! {};
// }

// Example 5: Missing equals sign
// #[test]
// fn missing_equals() {
//     let g = grammar! {
//         expr ::= term;
//     };
// }

#[test]
fn valid_grammar_compiles() {
    let g = grammar! {
        expr ::= term;
        term ::= digit;
        digit ::= '0'..'9';
    };
    assert_eq!(g.rules.len(), 3);
}
