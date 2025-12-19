//! AST node types and tree manipulation.

use crate::ebnf::Span;

/// A node in the abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    /// A terminal token (literal character or string).
    Terminal { value: String, span: Span },
    /// A sequence of nodes.
    Sequence { nodes: Vec<AstNode>, span: Span },
    /// An alternation (one of several alternatives matched).
    Alternation { nodes: Vec<AstNode>, span: Span },
    /// A repetition of nodes.
    Repetition { nodes: Vec<AstNode>, span: Span },
    /// A rule reference application.
    Rule {
        name: String,
        node: Box<AstNode>,
        span: Span,
    },
}

impl AstNode {
    /// Get the span of this node.
    pub fn span(&self) -> Span {
        match self {
            Self::Terminal { span, .. }
            | Self::Sequence { span, .. }
            | Self::Alternation { span, .. }
            | Self::Repetition { span, .. }
            | Self::Rule { span, .. } => *span,
        }
    }

    /// Convert node to a string representation (for debugging).
    pub fn to_string_debug(&self) -> String {
        match self {
            Self::Terminal { value, .. } => format!("Terminal({})", value),
            Self::Sequence { nodes, .. } => {
                let inner = nodes
                    .iter()
                    .map(|n| n.to_string_debug())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Sequence({})", inner)
            }
            Self::Alternation { nodes, .. } => {
                let inner = nodes
                    .iter()
                    .map(|n| n.to_string_debug())
                    .collect::<Vec<_>>()
                    .join("|");
                format!("Alternation({})", inner)
            }
            Self::Repetition { nodes, .. } => {
                let inner = nodes
                    .iter()
                    .map(|n| n.to_string_debug())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Repetition({})", inner)
            }
            Self::Rule { name, node, .. } => {
                format!("Rule({}: {})", name, node.to_string_debug())
            }
        }
    }
}

/// Complete abstract syntax tree for a parsed input.
#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    /// The root node of the tree.
    pub root: AstNode,
    /// Optional metadata about the parse (e.g., total span).
    pub metadata: AstMetadata,
}

/// Metadata about the AST.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AstMetadata {
    /// Total byte length of the input.
    pub input_length: usize,
    /// Number of tokens in the entire tree.
    pub token_count: usize,
    /// Whether the parse succeeded without errors.
    pub success: bool,
}

impl Ast {
    /// Get the total span of the AST.
    pub fn span(&self) -> Span {
        self.root.span()
    }

    /// Walk the tree and collect all terminals.
    pub fn collect_terminals(&self) -> Vec<String> {
        let mut terminals = Vec::new();
        self.walk_terminals(&self.root, &mut terminals);
        terminals
    }

    fn walk_terminals(&self, node: &AstNode, acc: &mut Vec<String>) {
        match node {
            AstNode::Terminal { value, .. } => acc.push(value.clone()),
            AstNode::Sequence { nodes, .. }
            | AstNode::Alternation { nodes, .. }
            | AstNode::Repetition { nodes, .. } => {
                for n in nodes {
                    self.walk_terminals(n, acc);
                }
            }
            AstNode::Rule { node, .. } => self.walk_terminals(node, acc),
        }
    }

    /// Get the depth of the tree.
    pub fn depth(&self) -> usize {
        self.node_depth(&self.root)
    }

    fn node_depth(&self, node: &AstNode) -> usize {
        match node {
            AstNode::Terminal { .. } => 1,
            AstNode::Sequence { nodes, .. }
            | AstNode::Alternation { nodes, .. }
            | AstNode::Repetition { nodes, .. } => {
                1 + nodes.iter().map(|n| self.node_depth(n)).max().unwrap_or(0)
            }
            AstNode::Rule { node, .. } => 1 + self.node_depth(node),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_terminal() {
        let span = Span::new(0, 5);
        let node = AstNode::Terminal {
            value: "hello".to_string(),
            span,
        };
        assert_eq!(node.span(), span);
    }

    #[test]
    fn test_ast_depth() {
        let mut builder = crate::ast::builder::AstBuilder::new();
        builder.push_sequence(Span::new(0, 10));
        builder.add_terminal("a".to_string(), Span::new(0, 1));
        builder.push_sequence(Span::new(1, 10));
        builder.add_terminal("b".to_string(), Span::new(1, 2));
        builder.pop_sequence();
        builder.pop_sequence();

        let ast = builder.build(2).unwrap();
        assert!(ast.depth() > 1);
    }
}
