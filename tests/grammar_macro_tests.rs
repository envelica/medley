use medley::ebnf::grammar;

#[test]
fn test_simple_grammar_compiles() {
    let g = grammar! {
        digit ::= '0'..'9';
        number ::= digit { digit };
    };

    assert_eq!(g.rules.len(), 2);
    assert_eq!(g.rules[0].name, "digit");
    assert_eq!(g.rules[1].name, "number");
}

#[test]
fn test_expression_grammar() {
    let g = grammar! {
        expr   ::= term { ('+' | '-') term };
        term   ::= factor { ('*' | '/') factor };
        factor ::= number | '(' expr ')';
        number ::= digit { digit };
        digit  ::= '0'..'9';
    };

    assert_eq!(g.rules.len(), 5);
    let errors = g.validate();
    assert!(errors.is_empty(), "validation errors: {:?}", errors);
}

#[test]
fn test_char_and_string_literals() {
    let g = grammar! {
        keyword ::= "if" | "else" | "while";
        op ::= '+' | '-' | '*' | '/';
    };

    assert_eq!(g.rules.len(), 2);
}

#[test]
fn test_numeric_character_references() {
    let g = grammar! {
        tab      ::= #x9;
        newline  ::= #10;
        capital_a ::= #X41;
    };

    assert_eq!(g.rules.len(), 3);
    let errors = g.validate();
    assert!(errors.is_empty(), "validation errors: {:?}", errors);
}
