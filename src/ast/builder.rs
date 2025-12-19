//! AST builder for constructing syntax trees from parse events.

use super::node::{Ast, AstMetadata, AstNode};
use crate::ebnf::Span;

/// Builder for constructing an AST from a sequence of parse events.
#[derive(Debug)]
pub struct AstBuilder {
    stack: Vec<Vec<AstNode>>,
    current_span_stack: Vec<Span>,
    rule_stack: Vec<String>,
    metadata: AstMetadata,
}

impl AstBuilder {
    /// Create a new AST builder.
    pub fn new() -> Self {
        Self {
            stack: vec![Vec::new()],
            current_span_stack: vec![],
            rule_stack: Vec::new(),
            metadata: AstMetadata::default(),
        }
    }

    /// Add a terminal node.
    pub fn add_terminal(&mut self, value: String, span: Span) {
        let node = AstNode::Terminal { value, span };
        self.stack.last_mut().unwrap().push(node);
        self.metadata.token_count += 1;
    }

    /// Push a new sequence context.
    pub fn push_sequence(&mut self, span: Span) {
        self.stack.push(Vec::new());
        self.current_span_stack.push(span);
    }

    /// Pop and finalize a sequence.
    pub fn pop_sequence(&mut self) -> Option<AstNode> {
        if self.stack.len() <= 1 {
            return None;
        }
        let nodes = self.stack.pop().unwrap();
        let span = self.current_span_stack.pop()?;

        let node = if nodes.is_empty() {
            AstNode::Sequence { nodes, span }
        } else if nodes.len() == 1 {
            nodes.into_iter().next().unwrap()
        } else {
            AstNode::Sequence { nodes, span }
        };

        self.stack.last_mut().unwrap().push(node.clone());
        Some(node)
    }

    /// Push a new alternation context.
    pub fn push_alternation(&mut self, span: Span) {
        self.stack.push(Vec::new());
        self.current_span_stack.push(span);
    }

    /// Pop and finalize an alternation.
    pub fn pop_alternation(&mut self) -> Option<AstNode> {
        if self.stack.len() <= 1 {
            return None;
        }
        let nodes = self.stack.pop().unwrap();
        let span = self.current_span_stack.pop()?;

        let node = if nodes.is_empty() {
            AstNode::Alternation { nodes, span }
        } else if nodes.len() == 1 {
            nodes.into_iter().next().unwrap()
        } else {
            AstNode::Alternation { nodes, span }
        };

        self.stack.last_mut().unwrap().push(node.clone());
        Some(node)
    }

    /// Push a repetition context.
    pub fn push_repetition(&mut self, span: Span) {
        self.stack.push(Vec::new());
        self.current_span_stack.push(span);
    }

    /// Pop and finalize a repetition.
    pub fn pop_repetition(&mut self) -> Option<AstNode> {
        if self.stack.len() <= 1 {
            return None;
        }
        let nodes = self.stack.pop().unwrap();
        let span = self.current_span_stack.pop()?;

        let node = AstNode::Repetition { nodes, span };
        self.stack.last_mut().unwrap().push(node.clone());
        Some(node)
    }

    /// Push a rule context.
    pub fn push_rule(&mut self, name: String) {
        self.rule_stack.push(name);
        self.stack.push(Vec::new());
    }

    /// Pop and finalize a rule.
    pub fn pop_rule(&mut self, span: Span) -> Option<AstNode> {
        let name = self.rule_stack.pop()?;
        if self.stack.len() <= 1 {
            return None;
        }
        let nodes = self.stack.pop().unwrap();

        let inner = if nodes.is_empty() {
            AstNode::Sequence { nodes, span }
        } else if nodes.len() == 1 {
            nodes.into_iter().next().unwrap()
        } else {
            AstNode::Sequence { nodes, span }
        };

        let node = AstNode::Rule {
            name,
            node: Box::new(inner),
            span,
        };
        self.stack.last_mut().unwrap().push(node.clone());
        Some(node)
    }

    /// Build the final AST. Returns an error if the builder state is invalid.
    pub fn build(mut self, input_length: usize) -> Result<Ast, String> {
        if self.stack.len() != 1 {
            return Err(format!(
                "Invalid builder state: stack has {} levels, expected 1",
                self.stack.len()
            ));
        }

        let nodes = self.stack.pop().unwrap();
        if nodes.is_empty() {
            return Err("No nodes in AST".to_string());
        }

        let root = if nodes.len() == 1 {
            nodes.into_iter().next().unwrap()
        } else {
            AstNode::Sequence {
                nodes,
                span: Span::new(0, input_length),
            }
        };

        self.metadata.input_length = input_length;
        self.metadata.success = true;

        Ok(Ast {
            root,
            metadata: self.metadata,
        })
    }
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_builder_simple() {
        let mut builder = AstBuilder::new();
        builder.add_terminal("test".to_string(), Span::new(0, 4));

        let ast = builder.build(4).unwrap();
        assert_eq!(ast.metadata.token_count, 1);
        assert!(ast.metadata.success);
    }

    #[test]
    fn test_ast_builder_sequence() {
        let mut builder = AstBuilder::new();
        builder.push_sequence(Span::new(0, 10));
        builder.add_terminal("a".to_string(), Span::new(0, 1));
        builder.add_terminal("b".to_string(), Span::new(1, 2));
        builder.pop_sequence();

        let ast = builder.build(2).unwrap();
        assert_eq!(ast.metadata.token_count, 2);
        assert!(matches!(ast.root, AstNode::Sequence { .. }));
    }

    #[test]
    fn test_ast_collect_terminals() {
        let mut builder = AstBuilder::new();
        builder.push_sequence(Span::new(0, 10));
        builder.add_terminal("a".to_string(), Span::new(0, 1));
        builder.add_terminal("b".to_string(), Span::new(1, 2));
        builder.pop_sequence();

        let ast = builder.build(2).unwrap();
        let terminals = ast.collect_terminals();
        assert_eq!(terminals, vec!["a", "b"]);
    }
}
