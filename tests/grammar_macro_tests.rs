use medley::ebnf::grammar;

#[test]
fn test_simple_grammar_compiles() {
    let g = grammar! {
        digit = [0-9];
        number = digit+;
    };
    
    assert_eq!(g.rules.len(), 2);
    assert_eq!(g.rules[0].name, "digit");
    assert_eq!(g.rules[1].name, "number");
}

#[test]
fn test_expression_grammar() {
    let g = grammar! {
        expr   = term (('+' | '-') term)*;
        term   = factor (('*' | '/') factor)*;
        factor = number | '(' expr ')';
        number = digit+;
        digit  = [0-9];
    };
    
    assert_eq!(g.rules.len(), 5);
    let errors = g.validate();
    assert!(errors.is_empty(), "validation errors: {:?}", errors);
}

#[test]
fn test_char_and_string_literals() {
    let g = grammar! {
        keyword = "if" | "else" | "while";
        op = '+' | '-' | '*' | '/';
    };
    
    assert_eq!(g.rules.len(), 2);
}