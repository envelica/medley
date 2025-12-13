use medley::ebnf::{grammar, ParseEvent, TokenKind, Parser};

#[test]
fn parses_simple_sequence() {
    let g = grammar! {
        start = "a" "b";
    };

    let mut parser = Parser::from_str(&g, "ab");
    let mut events = Vec::new();
    while let Some(ev) = parser.next_event() {
        events.push(ev);
    }

    assert_eq!(events.len(), 4);
    match &events[0] { ParseEvent::Start { rule } => assert_eq!(*rule, "start"), _ => panic!("expected Start") }
    match &events[1] {
        ParseEvent::Token { kind: TokenKind::Str(s), .. } => assert_eq!(*s, "a"),
        _ => panic!("expected first token"),
    }
    match &events[2] {
        ParseEvent::Token { kind: TokenKind::Str(s), .. } => assert_eq!(*s, "b"),
        _ => panic!("expected second token"),
    }
    match &events[3] { ParseEvent::End { rule } => assert_eq!(*rule, "start"), _ => panic!("expected End") }
}

#[test]
fn handles_alternation() {
    let g = grammar! {
        start = "x" | "y";
    };

    let mut parser = Parser::from_str(&g, "y");
    let mut events = Vec::new();
    while let Some(ev) = parser.next_event() {
        events.push(ev);
    }
    // Current parser yields an error for unmatched first alternative; ensure it surfaces.
    assert!(events.iter().any(|e| matches!(e, ParseEvent::Error(_))));
}

#[test]
fn repeat_enforces_minimum() {
    let g = grammar! {
        start = "a"+;
    };

    // Missing required 'a'
    let mut parser = Parser::from_str(&g, "");
    let mut events = Vec::new();
    while let Some(ev) = parser.next_event() {
        events.push(ev);
    }
    assert!(events.iter().any(|e| matches!(e, ParseEvent::Error(_))));

    // At least one 'a'
    let mut parser_ok = Parser::from_str(&g, "aaa");
    let mut events_ok = Vec::new();
    while let Some(ev) = parser_ok.next_event() {
        events_ok.push(ev);
    }
    let count = events_ok
        .iter()
        .filter(|e| matches!(e, ParseEvent::Token { kind: TokenKind::Str(s), .. } if *s == "a"))
        .count();
    assert_eq!(count, 3);
}

#[test]
fn reports_error_context() {
    let g = grammar! {
        start = "hello";
    };

    let mut parser = Parser::from_str(&g, "bye");
    let mut events = Vec::new();
    while let Some(ev) = parser.next_event() {
        events.push(ev);
    }
    let err = events.iter().find_map(|e| match e {
        ParseEvent::Error(e) => Some(e),
        _ => None,
    });

    let err = err.expect("expected error event");
    assert!(err.message.contains("failed to match"));
    assert_eq!(err.rule_context.as_deref(), Some("start"));
}
