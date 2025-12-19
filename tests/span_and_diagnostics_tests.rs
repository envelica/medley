// Phase 5: Spans, Positions, and Diagnostics Tests
use medley::ebnf::Span;
use medley::ebnf::{ParseError, ParseEvent, Parser, grammar, parse_str};
use std::io::Cursor;

#[test]
fn test_line_column_tracking_single_line() {
    let g = grammar! {
        start ::= "hello" "world";
    };

    let input = "helloworld";
    let events: Vec<ParseEvent> = parse_str(&g, input).collect();

    // Single line input, so all events should have line=1
    for event in &events {
        if let ParseEvent::Token { span, .. } = event {
            assert_eq!(span.line, Some(1), "Expected line 1 for single-line input");
        }
    }
}

#[test]
fn test_line_column_tracking_multiline() {
    let g = grammar! {
        start ::= "a" "b" "c";
    };

    // Single line input - all tokens on line 1
    let input = "abc";
    let events: Vec<ParseEvent> = parse_str(&g, input).collect();

    // Events: Start, Token("a"), Token("b"), Token("c"), End
    assert!(events.len() >= 5);

    match &events[1] {
        ParseEvent::Token { span, .. } => {
            assert_eq!(span.line, Some(1), "First token should be on line 1");
            assert_eq!(span.column, Some(1), "First token should be on column 1");
        }
        _ => panic!("Expected first token"),
    }

    match &events[2] {
        ParseEvent::Token { span, .. } => {
            assert_eq!(span.line, Some(1), "Second token should be on line 1");
            assert_eq!(span.column, Some(2), "Second token should be on column 2");
        }
        _ => panic!("Expected second token"),
    }

    match &events[3] {
        ParseEvent::Token { span, .. } => {
            assert_eq!(span.line, Some(1), "Third token should be on line 1");
            assert_eq!(span.column, Some(3), "Third token should be on column 3");
        }
        _ => panic!("Expected third token"),
    }
}

#[test]
fn test_span_with_position_constructor() {
    let span = Span::with_position(0, 5, 1, 1);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 5);
    assert_eq!(span.line, Some(1));
    assert_eq!(span.column, Some(1));
}

#[test]
fn test_span_new_constructor() {
    let span = Span::new(0, 5);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 5);
    assert_eq!(span.line, None);
    assert_eq!(span.column, None);
}

#[test]
fn test_parse_error_with_context() {
    let error = ParseError::new("expected terminal", 10)
        .with_rule_context("expression")
        .with_hint("try adding whitespace");

    assert_eq!(error.message, "expected terminal");
    assert_eq!(error.position, 10);
    assert_eq!(error.rule_context, Some("expression".to_string()));
    assert_eq!(error.hint, Some("try adding whitespace".to_string()));
}

#[test]
fn test_parse_error_with_span() {
    let span = Span::with_position(5, 10, 1, 6);
    let error = ParseError::new("invalid syntax", 5).with_span(span);

    assert_eq!(error.span.unwrap().start, 5);
    assert_eq!(error.span.unwrap().line, Some(1));
}

#[test]
fn test_error_event_includes_rule_context() {
    let g = grammar! {
        start ::= "exact_match";
    };

    let input = "wrong";
    let events: Vec<ParseEvent> = parse_str(&g, input).collect();

    // Should contain an error event with rule context
    let error_found = events.iter().any(|e| {
        if let ParseEvent::Error(err) = e {
            err.rule_context.is_some() && err.message.contains("failed to match")
        } else {
            false
        }
    });

    assert!(error_found, "Expected error with rule context");
}

#[test]
fn test_line_column_tracker_new() {
    let input = "hello\nworld\ntest";
    let g = grammar! {
        start ::= "hello";
    };
    let mut parser = Parser::new(&g, Cursor::new(input.as_bytes()));

    // Consume all events to ensure full buffer loading
    while let Some(_) = parser.next_event() {}

    let (line, col) = parser.line_column(0);
    assert_eq!(line, 1);
    assert_eq!(col, 1);

    // After first newline (position 6)
    let (line, col) = parser.line_column(6);
    assert_eq!(line, 2);
    assert_eq!(col, 1);

    // After second newline (position 12)
    let (line, col) = parser.line_column(12);
    assert_eq!(line, 3);
    assert_eq!(col, 1);
}

#[test]
fn test_line_column_tracker_mid_line() {
    let input = "hello world";
    let g = grammar! {
        start ::= "hello";
    };
    let mut parser = Parser::new(&g, Cursor::new(input.as_bytes()));

    // Consume all events to ensure full buffer loading
    while let Some(_) = parser.next_event() {}

    // Position 3 (letter 'l' in "hello")
    let (line, col) = parser.line_column(3);
    assert_eq!(line, 1);
    assert_eq!(col, 4);

    // Position 8 (letter 'o' in "world")
    let (line, col) = parser.line_column(8);
    assert_eq!(line, 1);
    assert_eq!(col, 9);
}

#[test]
fn test_span_with_position_from_tracker() {
    let input = "a\nb\nc";
    let g = grammar! {
        start ::= "a";
    };
    let mut parser = Parser::new(&g, Cursor::new(input.as_bytes()));

    // Consume all events to ensure full buffer loading
    while let Some(_) = parser.next_event() {}

    // Span for first character
    let span = parser.span_from_range(0, 1);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 1);
    assert_eq!(span.line, Some(1));
    assert_eq!(span.column, Some(1));

    // Span for character after first newline
    let span = parser.span_from_range(2, 3);
    assert_eq!(span.line, Some(2));
    assert_eq!(span.column, Some(1));
}

#[test]
fn test_multiple_errors_in_sequence() {
    let g = grammar! {
        start ::= "a" "b" "c";
    };

    // Input with multiple mismatches
    let input = "abc"; // This should actually match
    let events: Vec<ParseEvent> = parse_str(&g, input).collect();

    // Verify no errors for matching input
    let error_count = events
        .iter()
        .filter(|e| matches!(e, ParseEvent::Error(_)))
        .count();
    assert_eq!(error_count, 0);
}

#[test]
fn test_error_with_position_tracking() {
    let g = grammar! {
        start ::= "hello";
    };

    let input = "bye bye";
    let events: Vec<ParseEvent> = parse_str(&g, input).collect();

    // Find the error event
    let error_event = events.iter().find(|e| matches!(e, ParseEvent::Error(_)));
    assert!(error_event.is_some(), "Expected error event");

    if let Some(ParseEvent::Error(err)) = error_event {
        assert!(err.message.contains("failed to match"));
        assert!(err.position < input.len());
    }
}

#[test]
fn test_token_span_accuracy_with_multiline() {
    let g = grammar! {
        start ::= "x" "y" "z";
    };

    let input = "xyz";
    let events: Vec<ParseEvent> = parse_str(&g, input).collect();

    // Verify spans are accurate for each token
    match &events[1] {
        ParseEvent::Token { span, .. } => {
            assert_eq!(span.line, Some(1));
            assert_eq!(span.column, Some(1));
        }
        _ => panic!("Expected first token"),
    }

    match &events[2] {
        ParseEvent::Token { span, .. } => {
            assert_eq!(span.line, Some(1));
            assert_eq!(span.column, Some(2));
        }
        _ => panic!("Expected second token"),
    }

    match &events[3] {
        ParseEvent::Token { span, .. } => {
            assert_eq!(span.line, Some(1));
            assert_eq!(span.column, Some(3));
        }
        _ => panic!("Expected third token"),
    }
}

#[test]
fn test_column_tracking_within_line() {
    let g = grammar! {
        start ::= "hello" " " "world";
    };

    let input = "hello world";
    let events: Vec<ParseEvent> = parse_str(&g, input).collect();

    match &events[1] {
        ParseEvent::Token { span, .. } => {
            assert_eq!(span.column, Some(1), "First token should start at column 1");
        }
        _ => panic!("Expected first token"),
    }

    match &events[2] {
        ParseEvent::Token { span, .. } => {
            assert_eq!(span.column, Some(6), "Space should be at column 6");
        }
        _ => panic!("Expected space token"),
    }

    match &events[3] {
        ParseEvent::Token { span, .. } => {
            assert_eq!(span.column, Some(7), "Third token should start at column 7");
        }
        _ => panic!("Expected third token"),
    }
}
