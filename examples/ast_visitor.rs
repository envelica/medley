// Example demonstrating the Visitor pattern for AST traversal
use medley::ast::{Visitor, parse_str};
use medley::ebnf::{grammar, Span};

// Example 1: Count all terminals in the AST
struct TerminalCounter {
    count: usize,
}

impl Visitor for TerminalCounter {
    fn visit_terminal(&mut self, _value: &str, _span: &Span) {
        self.count += 1;
    }
}

// Example 2: Collect all rule names
struct RuleCollector {
    rules: Vec<String>,
}

impl Visitor for RuleCollector {
    fn visit_rule(&mut self, name: &str, node: &medley::ast::AstNode, _span: &Span) {
        self.rules.push(name.to_string());
        // Continue traversing into the rule's content
        self.visit_node(node);
    }
}

// Example 3: Build a string representation
struct AstPrinter {
    output: String,
    indent: usize,
}

impl AstPrinter {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    fn indent_str(&self) -> String {
        "  ".repeat(self.indent)
    }
}

impl Visitor for AstPrinter {
    fn visit_terminal(&mut self, value: &str, _span: &Span) {
        self.output.push_str(&format!("{}Terminal: '{}'\n", self.indent_str(), value));
    }

    fn visit_sequence(&mut self, nodes: &[medley::ast::AstNode], _span: &Span) {
        self.output.push_str(&format!("{}Sequence:\n", self.indent_str()));
        self.indent += 1;
        for node in nodes {
            self.visit_node(node);
        }
        self.indent -= 1;
    }

    fn visit_rule(&mut self, name: &str, node: &medley::ast::AstNode, _span: &Span) {
        self.output.push_str(&format!("{}Rule '{}':\n", self.indent_str(), name));
        self.indent += 1;
        self.visit_node(node);
        self.indent -= 1;
    }
}

fn main() {
    let grammar = grammar! {
        expr = num op num;
        num = digit digit?;
        digit = [0-9];
        op = [+-];
    };

    let input = "12+34";
    let ast = parse_str(&grammar, input).expect("parse failed");

    // Example 1: Count terminals
    let mut counter = TerminalCounter { count: 0 };
    counter.visit_ast(&ast);
    println!("Terminal count: {}", counter.count);

    // Example 2: Collect rule names
    let mut collector = RuleCollector { rules: Vec::new() };
    collector.visit_ast(&ast);
    println!("\nRules visited: {:?}", collector.rules);

    // Example 3: Print AST structure
    let mut printer = AstPrinter::new();
    printer.visit_ast(&ast);
    println!("\nAST Structure:\n{}", printer.output);
}
