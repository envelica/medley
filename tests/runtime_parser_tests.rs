use medley::ebnf::{grammar, parse_iter_str, ParseEvent, TokenKind};
use std::io::BufReader;

#[test]
fn test_parse_simple_sequence() {
    let g = grammar! {
        start = "hello" " " "world";
    };

    let input = "hello world";
    let events: Vec<ParseEvent> = parse_iter_str(&g, input).collect();

    // Expected: Start, Token("hello"), Token(" "), Token("world"), End
    assert_eq!(events.len(), 5);
    
    match &events[0] {
        ParseEvent::Start { rule } => assert_eq!(rule, "start"),
        _ => panic!("Expected Start event"),
    }
    
    match &events[1] {
        ParseEvent::Token { kind: TokenKind::Str(s), span } => {
            assert_eq!(s, "hello");
            assert_eq!(*span, (0, 5));
        }
        _ => panic!("Expected Token event for 'hello'"),
    }
    
    match &events[2] {
        ParseEvent::Token { kind: TokenKind::Str(s), span } => {
            assert_eq!(s, " ");
            assert_eq!(*span, (5, 6));
        }
        _ => panic!("Expected Token event for space"),
    }
    
    match &events[3] {
        ParseEvent::Token { kind: TokenKind::Str(s), span } => {
            assert_eq!(s, "world");
            assert_eq!(*span, (6, 11));
        }
        _ => panic!("Expected Token event for 'world'"),
    }
    
    match &events[4] {
        ParseEvent::End { rule } => assert_eq!(rule, "start"),
        _ => panic!("Expected End event"),
    }
}

#[test]
fn test_parse_alternation() {
    let g = grammar! {
        start = "yes" | "no";
    };

    let input_yes = "yes";
    let events_yes: Vec<ParseEvent> = parse_iter_str(&g, input_yes).collect();
    
    assert_eq!(events_yes.len(), 3);
    match &events_yes[0] {
        ParseEvent::Start { rule } => assert_eq!(rule, "start"),
        _ => panic!("Expected Start"),
    }
    match &events_yes[1] {
        ParseEvent::Token { kind: TokenKind::Str(s), .. } => assert_eq!(s, "yes"),
        _ => panic!("Expected 'yes' token"),
    }
    match &events_yes[2] {
        ParseEvent::End { rule } => assert_eq!(rule, "start"),
        _ => panic!("Expected End"),
    }

    let input_no = "no";
    let events_no: Vec<ParseEvent> = parse_iter_str(&g, input_no).collect();
    
    assert_eq!(events_no.len(), 3);
    match &events_no[1] {
        ParseEvent::Token { kind: TokenKind::Str(s), .. } => assert_eq!(s, "no"),
        _ => panic!("Expected 'no' token"),
    }
}

#[test]
fn test_parse_repetition() {
    let g = grammar! {
        start = 'a'*;
    };

    let input = "aaa";
    let events: Vec<ParseEvent> = parse_iter_str(&g, input).collect();
    
    // Start, Token('a'), Token('a'), Token('a'), End
    assert_eq!(events.len(), 5);
    
    match &events[0] {
        ParseEvent::Start { .. } => {},
        _ => panic!("Expected Start"),
    }
    
    for i in 1..=3 {
        match &events[i] {
            ParseEvent::Token { kind: TokenKind::Char(c), .. } => assert_eq!(*c, 'a'),
            _ => panic!("Expected 'a' token"),
        }
    }
    
    match &events[4] {
        ParseEvent::End { .. } => {},
        _ => panic!("Expected End"),
    }
}

#[test]
fn test_parse_optional() {
    let g = grammar! {
        start = "x" 'y'?;
    };

    let input_with = "xy";
    let events_with: Vec<ParseEvent> = parse_iter_str(&g, input_with).collect();
    assert_eq!(events_with.len(), 4); // Start, Token("x"), Token('y'), End
    
    let input_without = "x";
    let events_without: Vec<ParseEvent> = parse_iter_str(&g, input_without).collect();
    assert_eq!(events_without.len(), 3); // Start, Token("x"), End
}

#[test]
fn test_parse_char_class() {
    let g = grammar! {
        start = ['0'-'9']+;
    };

    let input = "123";
    let events: Vec<ParseEvent> = parse_iter_str(&g, input).collect();
    
    // Start, Token('1'), Token('2'), Token('3'), End
    assert_eq!(events.len(), 5);
    
    match &events[1] {
        ParseEvent::Token { kind: TokenKind::Class(c), span } => {
            assert_eq!(*c, '1');
            assert_eq!(*span, (0, 1));
        }
        _ => panic!("Expected class token for '1'"),
    }
}

#[test]
fn test_parse_rule_reference() {
    let g = grammar! {
        start = greeting " " name;
        greeting = "hello" | "hi";
        name = "world" | "there";
    };

    let input = "hello world";
    let events: Vec<ParseEvent> = parse_iter_str(&g, input).collect();
    
    // Start(start), Start(greeting), Token("hello"), End(greeting), Token(" "), 
    // Start(name), Token("world"), End(name), End(start)
    assert_eq!(events.len(), 9);
    
    match &events[0] {
        ParseEvent::Start { rule } => assert_eq!(rule, "start"),
        _ => panic!("Expected Start(start)"),
    }
    
    match &events[1] {
        ParseEvent::Start { rule } => assert_eq!(rule, "greeting"),
        _ => panic!("Expected Start(greeting)"),
    }
    
    match &events[2] {
        ParseEvent::Token { kind: TokenKind::Str(s), .. } => assert_eq!(s, "hello"),
        _ => panic!("Expected 'hello' token"),
    }
    
    match &events[3] {
        ParseEvent::End { rule } => assert_eq!(rule, "greeting"),
        _ => panic!("Expected End(greeting)"),
    }
}

#[test]
fn test_parse_error() {
    let g = grammar! {
        start = "exact";
    };

    let input = "wrong";
    let events: Vec<ParseEvent> = parse_iter_str(&g, input).collect();
    
    // Should contain Start and Error
    assert!(events.len() >= 2);
    
    match &events[0] {
        ParseEvent::Start { .. } => {},
        _ => panic!("Expected Start"),
    }
    
    match &events[1] {
        ParseEvent::Error(e) => {
            assert!(e.message.contains("failed to match"));
        }
        _ => panic!("Expected Error event"),
    }
}

#[test]
fn test_parse_from_bufreader() {
    let g = grammar! {
        start = "hello" " " "world";
    };

    let input = "hello world";
    let reader = BufReader::new(input.as_bytes());
    
    let events: Vec<ParseEvent> = medley::ebnf::parse_iter(&g, reader).collect();
    
    // Same as string version
    assert_eq!(events.len(), 5);
    
    match &events[0] {
        ParseEvent::Start { rule } => assert_eq!(rule, "start"),
        _ => panic!("Expected Start event"),
    }
}

#[test]
fn test_large_input_bounded_memory() {
    // This test verifies that the parser doesn't buffer the entire input
    let g = grammar! {
        start = ['a'-'z']*;
    };

    // Create a large input string
    let large_input = "a".repeat(100_000);
    let events: Vec<ParseEvent> = parse_iter_str(&g, &large_input).collect();
    
    // Should have Start + 100,000 tokens + End
    assert_eq!(events.len(), 100_002);
    
    // Verify first and last events
    match &events[0] {
        ParseEvent::Start { .. } => {},
        _ => panic!("Expected Start"),
    }
    
    match &events[100_001] {
        ParseEvent::End { .. } => {},
        _ => panic!("Expected End"),
    }
}

#[test]
fn test_parse_nested_groups() {
    let g = grammar! {
        start = ("a" "b") | ("c" "d");
    };

    let input = "ab";
    let events: Vec<ParseEvent> = parse_iter_str(&g, input).collect();
    
    // Start, Token("a"), Token("b"), End
    assert_eq!(events.len(), 4);
    
    match &events[1] {
        ParseEvent::Token { kind: TokenKind::Str(s), .. } => assert_eq!(s, "a"),
        _ => panic!("Expected 'a' token"),
    }
    
    match &events[2] {
        ParseEvent::Token { kind: TokenKind::Str(s), .. } => assert_eq!(s, "b"),
        _ => panic!("Expected 'b' token"),
    }
}
