// Phase 8: Unit tests for Grammar IR validation
use medley::ebnf::{CharClass, Grammar, Prod, RepeatQuant, Rule, TerminalKind};

#[test]
fn test_valid_simple_grammar() {
    let g = Grammar {
        rules: vec![Rule {
            name: "start".to_string(),
            production: Prod::Terminal {
                kind: TerminalKind::Str("hello".to_string()),
                span: None,
            },
            span: None,
        }],
    };

    let errors = g.validate();
    assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
}

#[test]
fn test_undefined_rule_reference() {
    let g = Grammar {
        rules: vec![Rule {
            name: "start".to_string(),
            production: Prod::Ref {
                name: "undefined".to_string(),
                span: None,
            },
            span: None,
        }],
    };

    let errors = g.validate();
    assert!(
        !errors.is_empty(),
        "Expected errors for undefined reference"
    );
    assert!(
        errors.iter().any(|e| e.contains("undefined")),
        "Expected 'undefined' in error messages: {:?}",
        errors
    );
}

#[test]
fn test_empty_grammar() {
    let g = Grammar { rules: vec![] };

    let errors = g.validate();
    assert!(!errors.is_empty(), "Expected error for empty grammar");
}

#[test]
fn test_direct_left_recursion() {
    let g = Grammar {
        rules: vec![Rule {
            name: "expr".to_string(),
            production: Prod::Seq(vec![
                Prod::Ref {
                    name: "expr".to_string(),
                    span: None,
                },
                Prod::Terminal {
                    kind: TerminalKind::Char('+'),
                    span: None,
                },
                Prod::Terminal {
                    kind: TerminalKind::Char('1'),
                    span: None,
                },
            ]),
            span: None,
        }],
    };

    let errors = g.validate();
    assert!(!errors.is_empty(), "Expected error for left-recursion");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("left") || e.contains("recurs")),
        "Expected left-recursion error: {:?}",
        errors
    );
}

#[test]
fn test_indirect_left_recursion() {
    let g = Grammar {
        rules: vec![
            Rule {
                name: "a".to_string(),
                production: Prod::Ref {
                    name: "b".to_string(),
                    span: None,
                },
                span: None,
            },
            Rule {
                name: "b".to_string(),
                production: Prod::Ref {
                    name: "a".to_string(),
                    span: None,
                },
                span: None,
            },
        ],
    };

    let errors = g.validate();
    assert!(!errors.is_empty(), "Expected error for cyclic dependency");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("cyclic") || e.contains("recurs") || e.contains("indirect")),
        "Expected cyclic dependency error: {:?}",
        errors
    );
}

#[test]
fn test_empty_sequence() {
    let g = Grammar {
        rules: vec![Rule {
            name: "start".to_string(),
            production: Prod::Seq(vec![]),
            span: None,
        }],
    };

    let errors = g.validate();
    assert!(
        errors.is_empty(),
        "Empty sequence should be valid: {:?}",
        errors
    );
}

#[test]
fn test_empty_alternation() {
    let g = Grammar {
        rules: vec![Rule {
            name: "start".to_string(),
            production: Prod::Alt(vec![]),
            span: None,
        }],
    };

    let errors = g.validate();
    assert!(
        errors.is_empty(),
        "Empty alternation should be valid: {:?}",
        errors
    );
}

#[test]
fn test_nested_references() {
    let g = Grammar {
        rules: vec![
            Rule {
                name: "start".to_string(),
                production: Prod::Ref {
                    name: "a".to_string(),
                    span: None,
                },
                span: None,
            },
            Rule {
                name: "a".to_string(),
                production: Prod::Ref {
                    name: "b".to_string(),
                    span: None,
                },
                span: None,
            },
            Rule {
                name: "b".to_string(),
                production: Prod::Terminal {
                    kind: TerminalKind::Char('x'),
                    span: None,
                },
                span: None,
            },
        ],
    };

    let errors = g.validate();
    assert!(
        errors.is_empty(),
        "Nested references should be valid: {:?}",
        errors
    );
}

#[test]
fn test_valid_repetition_quantifiers() {
    let g = Grammar {
        rules: vec![Rule {
            name: "start".to_string(),
            production: Prod::Seq(vec![
                Prod::Repeat {
                    item: Box::new(Prod::Terminal {
                        kind: TerminalKind::Char('a'),
                        span: None,
                    }),
                    quant: RepeatQuant { min: 0, max: None },
                },
                Prod::Repeat {
                    item: Box::new(Prod::Terminal {
                        kind: TerminalKind::Char('b'),
                        span: None,
                    }),
                    quant: RepeatQuant { min: 1, max: None },
                },
                Prod::Repeat {
                    item: Box::new(Prod::Terminal {
                        kind: TerminalKind::Char('c'),
                        span: None,
                    }),
                    quant: RepeatQuant {
                        min: 0,
                        max: Some(1),
                    },
                },
            ]),
            span: None,
        }],
    };

    let errors = g.validate();
    assert!(
        errors.is_empty(),
        "Repetition quantifiers should be valid: {:?}",
        errors
    );
}

#[test]
fn test_character_class_ranges() {
    let g = Grammar {
        rules: vec![Rule {
            name: "start".to_string(),
            production: Prod::Class(CharClass {
                negated: false,
                chars: vec![],
                ranges: vec![('a', 'z'), ('A', 'Z'), ('0', '9')],
                span: None,
            }),
            span: None,
        }],
    };

    let errors = g.validate();
    assert!(
        errors.is_empty(),
        "Character class ranges should be valid: {:?}",
        errors
    );
}

#[test]
fn test_negated_character_class() {
    let g = Grammar {
        rules: vec![Rule {
            name: "start".to_string(),
            production: Prod::Class(CharClass {
                negated: true,
                chars: vec![],
                ranges: vec![('0', '9')],
                span: None,
            }),
            span: None,
        }],
    };

    let errors = g.validate();
    assert!(
        errors.is_empty(),
        "Negated character class should be valid: {:?}",
        errors
    );
}

#[test]
fn test_complex_alternation() {
    let g = Grammar {
        rules: vec![
            Rule {
                name: "start".to_string(),
                production: Prod::Alt(vec![
                    Prod::Terminal {
                        kind: TerminalKind::Str("option1".to_string()),
                        span: None,
                    },
                    Prod::Terminal {
                        kind: TerminalKind::Str("option2".to_string()),
                        span: None,
                    },
                    Prod::Ref {
                        name: "nested".to_string(),
                        span: None,
                    },
                ]),
                span: None,
            },
            Rule {
                name: "nested".to_string(),
                production: Prod::Terminal {
                    kind: TerminalKind::Str("nested_option".to_string()),
                    span: None,
                },
                span: None,
            },
        ],
    };

    let errors = g.validate();
    assert!(
        errors.is_empty(),
        "Complex alternation should be valid: {:?}",
        errors
    );
}

#[test]
fn test_grouped_production() {
    let g = Grammar {
        rules: vec![Rule {
            name: "start".to_string(),
            production: Prod::Group(Box::new(Prod::Alt(vec![
                Prod::Terminal {
                    kind: TerminalKind::Char('a'),
                    span: None,
                },
                Prod::Terminal {
                    kind: TerminalKind::Char('b'),
                    span: None,
                },
            ]))),
            span: None,
        }],
    };

    let errors = g.validate();
    assert!(
        errors.is_empty(),
        "Grouped production should be valid: {:?}",
        errors
    );
}

#[test]
fn test_duplicate_rule_names() {
    let g = Grammar {
        rules: vec![
            Rule {
                name: "start".to_string(),
                production: Prod::Terminal {
                    kind: TerminalKind::Char('a'),
                    span: None,
                },
                span: None,
            },
            Rule {
                name: "start".to_string(),
                production: Prod::Terminal {
                    kind: TerminalKind::Char('b'),
                    span: None,
                },
                span: None,
            },
        ],
    };

    // Test documents current behavior with duplicate rules
    let errors = g.validate();
    println!("Duplicate rule validation: {:?}", errors);
}
