// Example demonstrating the mutable Visitor pattern for AST transformation
use medley::ast::{VisitorMut, parse_str};
use medley::ebnf::{grammar, Span};

// Example: Transform all terminal values to uppercase
struct Uppercaser;

impl VisitorMut for Uppercaser {
    fn visit_terminal_mut(&mut self, value: &mut String, _span: &mut Span) {
        *value = value.to_uppercase();
    }
}

// Example: Prefix all rule names
struct RulePrefixer {
    prefix: String,
}

impl VisitorMut for RulePrefixer {
    fn visit_rule_mut(&mut self, name: &mut String, node: &mut medley::ast::AstNode, _span: &mut Span) {
        *name = format!("{}{}", self.prefix, name);
        // Continue into the node
        self.visit_node_mut(node);
    }
}

fn main() {
    let grammar = grammar! {
        start = "hello" " " "world";
    };

    let input = "hello world";
    let mut ast = parse_str(&grammar, input).expect("parse failed");

    println!("Original terminals: {:?}", ast.collect_terminals());

    // Transform to uppercase
    let mut uppercaser = Uppercaser;
    uppercaser.visit_ast_mut(&mut ast);

    println!("Uppercased terminals: {:?}", ast.collect_terminals());

    // You can chain transformations
    let mut prefixer = RulePrefixer {
        prefix: "transformed_".to_string(),
    };
    prefixer.visit_ast_mut(&mut ast);

    println!("\nAST after transformations:");
    println!("  Terminals: {:?}", ast.collect_terminals());
    println!("  Root span: {:?}", ast.span());
}
