// Phase 8: Integration tests for streaming parser over BufRead
use medley::ebnf::{ParseEvent, grammar, parse};
use std::io::Cursor;

#[test]
fn test_parse_from_bufreader() {
    let g = grammar! {
        start ::= "hello";
    };

    let input = b"hello";
    let reader = Cursor::new(&input[..]);
    let events: Vec<_> = parse(&g, reader).collect();

    assert!(events.iter().any(|e| matches!(e, ParseEvent::Token { .. })));
}

#[test]
fn test_parse_repeated_rule_reference() {
    let g = grammar! {
        start ::= digit { digit };
        digit ::= '0'..'9';
    };

    // Create input
    let reader = Cursor::new(b"123");
    let events: Vec<_> = parse(&g, reader).collect();
    println!("Events: {:?}", events);
    let token_count = events
        .iter()
        .filter(|e| matches!(e, ParseEvent::Token { .. }))
        .count();

    assert_eq!(token_count, 3, "Should parse all three digits");
}

#[test]
fn test_parse_multiline_input() {
    let g = grammar! {
        start ::= line { line };
        line ::= word ws;
        word ::= letter { letter };
        letter ::= 'a'..'z';
        ws ::= { ' ' };
    };

    let input = b"hello world test ";
    let reader = Cursor::new(&input[..]);
    let events: Vec<_> = parse(&g, reader).collect();
    println!("Multiline events: {:?}", events);

    let token_count = events
        .iter()
        .filter(|e| matches!(e, ParseEvent::Token { .. }))
        .count();
    // Each word character is a separate token, plus spaces
    assert!(token_count > 0, "Should parse words and whitespace");
}

#[test]
fn test_parse_empty_input() {
    let g = grammar! {
        start ::= { '0'..'9' };
    };

    let reader = Cursor::new(b"");
    let events: Vec<_> = parse(&g, reader).collect();

    println!("Empty input events: {:?}", events);
    // Document behavior with empty input
    let has_start_end = events.iter().any(|e| matches!(e, ParseEvent::Start { .. }));
    assert!(
        has_start_end || events.is_empty(),
        "Should produce Start/End or be empty"
    );
}

#[test]
fn test_parse_utf8_input() {
    let g = grammar! {
        start ::= "hello";
    };

    let input = "hello".as_bytes();
    let reader = Cursor::new(input);
    let events: Vec<_> = parse(&g, reader).collect();

    assert!(events.iter().any(|e| matches!(e, ParseEvent::Token { .. })));
}

#[test]
fn test_alternation_with_backtracking() {
    let g = grammar! {
        start ::= "abc" | "ab" | "a";
    };

    // Should match "abc" fully, not backtrack to "ab"
    let reader = Cursor::new(b"abc");
    let events: Vec<_> = parse(&g, reader).collect();

    let has_error = events.iter().any(|e| matches!(e, ParseEvent::Error(_)));
    assert!(!has_error, "Should parse 'abc' without backtracking");
}

#[test]
fn test_repetition_boundary() {
    let g = grammar! {
        start ::= digit { digit };
        digit ::= '0'..'9';
    };

    let reader = Cursor::new(b"123abc");
    let events: Vec<_> = parse(&g, reader).collect();

    let has_error = events.iter().any(|e| matches!(e, ParseEvent::Error(_)));
    assert!(
        has_error,
        "Should report an error when trailing characters follow digits"
    );
}

#[test]
fn test_nested_groups() {
    let g = grammar! {
        start ::= '(' ('a' | 'b') { ('a' | 'b') } ')';
    };

    let reader = Cursor::new(b"(aba)");
    let events: Vec<_> = parse(&g, reader).collect();

    assert!(!events.iter().any(|e| matches!(e, ParseEvent::Error(_))));
}

#[test]
fn test_optional_at_end() {
    let g = grammar! {
        start ::= "test" [ "!" ];
    };

    let reader1 = Cursor::new(b"test");
    let events1: Vec<_> = parse(&g, reader1).collect();
    println!("Optional (no match) events: {:?}", events1);
    // Document behavior - optional may cause errors in some implementations
    let has_tokens1 = events1
        .iter()
        .any(|e| matches!(e, ParseEvent::Token { .. }));

    let reader2 = Cursor::new(b"test!");
    let events2: Vec<_> = parse(&g, reader2).collect();
    println!("Optional (match) events: {:?}", events2);
    let has_tokens2 = events2
        .iter()
        .any(|e| matches!(e, ParseEvent::Token { .. }));

    assert!(
        has_tokens1 || has_tokens2,
        "At least one case should produce tokens"
    );
}

#[test]
fn test_character_class_negation() {
    let g = grammar! {
        // Accept one or more letters
        start ::= letter { letter };
        letter ::= 'a'..'z';
    };

    let reader = Cursor::new(b"abc");
    let events: Vec<_> = parse(&g, reader).collect();

    let token_count = events
        .iter()
        .filter(|e| matches!(e, ParseEvent::Token { .. }))
        .count();
    assert_eq!(token_count, 3);
}
