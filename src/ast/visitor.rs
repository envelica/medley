//! Visitor pattern for traversing AST nodes.
//!
//! The visitor pattern allows you to define custom operations on AST nodes
//! without modifying the AST structure itself. This is useful for tasks like:
//! - Collecting statistics
//! - Transforming the tree
//! - Generating code
//! - Static analysis
//!
//! # Example
//! ```
//! use medley::ast::{Visitor, AstNode, parse_str};
//! use medley::ebnf::grammar;
//!
//! // Define a visitor that counts terminals
//! struct TerminalCounter {
//!     count: usize,
//! }
//!
//! impl Visitor for TerminalCounter {
//!     fn visit_terminal(&mut self, _value: &str, _span: &medley::ebnf::Span) {
//!         self.count += 1;
//!     }
//! }
//!
//! let g = grammar! {
//!     start = "hello" " " "world";
//! };
//!
//! let ast = parse_str(&g, "hello world").expect("parse failed");
//! let mut counter = TerminalCounter { count: 0 };
//! counter.visit_ast(&ast);
//! assert_eq!(counter.count, 3);
//! ```

use super::node::{AstNode, Ast};
use crate::ebnf::Span;

/// Visitor trait for traversing AST nodes immutably.
///
/// Implement this trait to define custom operations that read from the AST.
/// The default implementations traverse the entire tree, calling the appropriate
/// visit methods for each node type.
pub trait Visitor {
    /// Visit the entire AST starting from the root.
    fn visit_ast(&mut self, ast: &Ast) {
        self.visit_node(&ast.root);
    }

    /// Visit any AST node and dispatch to the appropriate specific method.
    fn visit_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Terminal { value, span } => self.visit_terminal(value, span),
            AstNode::Sequence { nodes, span } => self.visit_sequence(nodes, span),
            AstNode::Alternation { nodes, span } => self.visit_alternation(nodes, span),
            AstNode::Repetition { nodes, span } => self.visit_repetition(nodes, span),
            AstNode::Rule { name, node, span } => self.visit_rule(name, node, span),
        }
    }

    /// Visit a terminal node.
    fn visit_terminal(&mut self, _value: &str, _span: &Span) {}

    /// Visit a sequence node. Default implementation visits all children.
    fn visit_sequence(&mut self, nodes: &[AstNode], _span: &Span) {
        for node in nodes {
            self.visit_node(node);
        }
    }

    /// Visit an alternation node. Default implementation visits all children.
    fn visit_alternation(&mut self, nodes: &[AstNode], _span: &Span) {
        for node in nodes {
            self.visit_node(node);
        }
    }

    /// Visit a repetition node. Default implementation visits all children.
    fn visit_repetition(&mut self, nodes: &[AstNode], _span: &Span) {
        for node in nodes {
            self.visit_node(node);
        }
    }

    /// Visit a rule node. Default implementation visits the inner node.
    fn visit_rule(&mut self, _name: &str, node: &AstNode, _span: &Span) {
        self.visit_node(node);
    }
}

/// Mutable visitor trait for transforming AST nodes.
///
/// Implement this trait to define operations that modify the AST.
/// Use this when you need to transform or rewrite parts of the tree.
///
/// # Example
/// ```
/// use medley::ast::{VisitorMut, AstNode};
/// use medley::ebnf::Span;
///
/// // Define a visitor that uppercases all terminal values
/// struct Uppercaser;
///
/// impl VisitorMut for Uppercaser {
///     fn visit_terminal_mut(&mut self, value: &mut String, _span: &mut Span) {
///         *value = value.to_uppercase();
///     }
/// }
/// ```
pub trait VisitorMut {
    /// Visit the entire AST starting from the root.
    fn visit_ast_mut(&mut self, ast: &mut Ast) {
        self.visit_node_mut(&mut ast.root);
    }

    /// Visit any AST node mutably and dispatch to the appropriate specific method.
    fn visit_node_mut(&mut self, node: &mut AstNode) {
        match node {
            AstNode::Terminal { value, span } => self.visit_terminal_mut(value, span),
            AstNode::Sequence { nodes, span } => self.visit_sequence_mut(nodes, span),
            AstNode::Alternation { nodes, span } => self.visit_alternation_mut(nodes, span),
            AstNode::Repetition { nodes, span } => self.visit_repetition_mut(nodes, span),
            AstNode::Rule { name, node, span } => self.visit_rule_mut(name, node, span),
        }
    }

    /// Visit a terminal node mutably.
    fn visit_terminal_mut(&mut self, _value: &mut String, _span: &mut Span) {}

    /// Visit a sequence node mutably. Default implementation visits all children.
    fn visit_sequence_mut(&mut self, nodes: &mut [AstNode], _span: &mut Span) {
        for node in nodes {
            self.visit_node_mut(node);
        }
    }

    /// Visit an alternation node mutably. Default implementation visits all children.
    fn visit_alternation_mut(&mut self, nodes: &mut [AstNode], _span: &mut Span) {
        for node in nodes {
            self.visit_node_mut(node);
        }
    }

    /// Visit a repetition node mutably. Default implementation visits all children.
    fn visit_repetition_mut(&mut self, nodes: &mut [AstNode], _span: &mut Span) {
        for node in nodes {
            self.visit_node_mut(node);
        }
    }

    /// Visit a rule node mutably. Default implementation visits the inner node.
    fn visit_rule_mut(&mut self, _name: &mut String, node: &mut AstNode, _span: &mut Span) {
        self.visit_node_mut(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ebnf::Span;

    #[test]
    fn test_visitor_counts_terminals() {
        struct TerminalCounter {
            count: usize,
        }

        impl Visitor for TerminalCounter {
            fn visit_terminal(&mut self, _value: &str, _span: &Span) {
                self.count += 1;
            }
        }

        let node = AstNode::Sequence {
            nodes: vec![
                AstNode::Terminal {
                    value: "a".to_string(),
                    span: Span::new(0, 1),
                },
                AstNode::Terminal {
                    value: "b".to_string(),
                    span: Span::new(1, 2),
                },
            ],
            span: Span::new(0, 2),
        };

        let ast = Ast {
            root: node,
            metadata: Default::default(),
        };

        let mut counter = TerminalCounter { count: 0 };
        counter.visit_ast(&ast);
        assert_eq!(counter.count, 2);
    }

    #[test]
    fn test_visitor_collects_values() {
        struct ValueCollector {
            values: Vec<String>,
        }

        impl Visitor for ValueCollector {
            fn visit_terminal(&mut self, value: &str, _span: &Span) {
                self.values.push(value.to_string());
            }
        }

        let node = AstNode::Sequence {
            nodes: vec![
                AstNode::Terminal {
                    value: "hello".to_string(),
                    span: Span::new(0, 5),
                },
                AstNode::Terminal {
                    value: "world".to_string(),
                    span: Span::new(6, 11),
                },
            ],
            span: Span::new(0, 11),
        };

        let ast = Ast {
            root: node,
            metadata: Default::default(),
        };

        let mut collector = ValueCollector { values: Vec::new() };
        collector.visit_ast(&ast);
        assert_eq!(collector.values, vec!["hello", "world"]);
    }

    #[test]
    fn test_visitor_mut_transforms() {
        struct Uppercaser;

        impl VisitorMut for Uppercaser {
            fn visit_terminal_mut(&mut self, value: &mut String, _span: &mut Span) {
                *value = value.to_uppercase();
            }
        }

        let node = AstNode::Sequence {
            nodes: vec![
                AstNode::Terminal {
                    value: "hello".to_string(),
                    span: Span::new(0, 5),
                },
                AstNode::Terminal {
                    value: "world".to_string(),
                    span: Span::new(6, 11),
                },
            ],
            span: Span::new(0, 11),
        };

        let mut ast = Ast {
            root: node,
            metadata: Default::default(),
        };

        let mut uppercaser = Uppercaser;
        uppercaser.visit_ast_mut(&mut ast);

        // Verify transformation
        let terminals = ast.collect_terminals();
        assert_eq!(terminals, vec!["HELLO", "WORLD"]);
    }

    #[test]
    fn test_visitor_nested_structures() {
        struct DepthCounter {
            max_depth: usize,
            current_depth: usize,
        }

        impl Visitor for DepthCounter {
            fn visit_sequence(&mut self, nodes: &[AstNode], _span: &Span) {
                self.current_depth += 1;
                if self.current_depth > self.max_depth {
                    self.max_depth = self.current_depth;
                }
                for node in nodes {
                    self.visit_node(node);
                }
                self.current_depth -= 1;
            }
        }

        let inner = AstNode::Sequence {
            nodes: vec![AstNode::Terminal {
                value: "x".to_string(),
                span: Span::new(0, 1),
            }],
            span: Span::new(0, 1),
        };

        let outer = AstNode::Sequence {
            nodes: vec![inner],
            span: Span::new(0, 1),
        };

        let ast = Ast {
            root: outer,
            metadata: Default::default(),
        };

        let mut counter = DepthCounter {
            max_depth: 0,
            current_depth: 0,
        };
        counter.visit_ast(&ast);
        assert_eq!(counter.max_depth, 2);
    }
}
