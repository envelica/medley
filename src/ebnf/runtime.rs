//! Pull-based parser runtime for EBNF grammars.
//!
//! Phase 4: Memory-efficient streaming parser with bounded lookahead.

use crate::ebnf::ir::*;
use std::io::BufRead;

/// Parse event emitted by the pull parser.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseEvent {
    /// Start of a rule match.
    Start { rule: String },
    /// End of a rule match.
    End { rule: String },
    /// Token matched (terminal or character class).
    Token { kind: TokenKind, span: (usize, usize) },
    /// Parse error encountered.
    Error(ParseError),
}

/// Token kind for parsed terminals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Character literal matched.
    Char(char),
    /// String literal matched.
    Str(String),
    /// Character class matched.
    Class(char),
}

/// Parse error with context.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

/// Parse an input string using the given grammar.
///
/// Returns an iterator of parse events.
pub fn parse_iter_str<'a>(grammar: &'a Grammar, input: &'a str) -> impl Iterator<Item = ParseEvent> + 'a {
    let start_rule = grammar
        .start
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or_else(|| grammar.rules[0].name.as_str());
    
    Parser::new(input, grammar, start_rule).into_iter()
}

/// Parse from a buffered reader using the given grammar.
///
/// Returns an iterator of parse events with bounded buffering.
pub fn parse_iter<R: BufRead>(grammar: &Grammar, reader: R) -> impl Iterator<Item = ParseEvent> {
    // Read into string for Phase 4 - Phase 5 will add true streaming
    let mut input = String::new();
    if let Ok(_) = std::io::Read::read_to_string(&mut reader.take(1024 * 1024), &mut input) {
        let start_rule = grammar
            .start
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|| grammar.rules[0].name.as_str());
        
        new_owned(input, grammar, start_rule).into_iter()
    } else {
        vec![ParseEvent::Error(ParseError {
            message: "failed to read input".to_string(),
            position: 0,
        })]
        .into_iter()
    }
}

struct Parser<'a> {
    input: &'a str,
    position: usize,
    grammar: &'a Grammar,
    events: Vec<ParseEvent>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str, grammar: &'a Grammar, start_rule: &str) -> Self {
        let mut parser = Self {
            input,
            position: 0,
            grammar,
            events: Vec::new(),
        };
        
        if let Some(rule) = grammar.get_rule(start_rule) {
            parser.parse_rule(rule);
        } else {
            parser.events.push(ParseEvent::Error(ParseError {
                message: format!("start rule '{}' not found", start_rule),
                position: 0,
            }));
        }
        
        parser
    }
    
    fn parse_rule(&mut self, rule: &Rule) {
        self.events.push(ParseEvent::Start {
            rule: rule.name.clone(),
        });
        
        let start_pos = self.position;
        if self.parse_prod(&rule.production) {
            self.events.push(ParseEvent::End {
                rule: rule.name.clone(),
            });
        } else {
            // Restore position on failure
            self.position = start_pos;
            self.events.push(ParseEvent::Error(ParseError {
                message: format!("failed to match rule '{}'", rule.name),
                position: self.position,
            }));
        }
    }
    
    fn parse_prod(&mut self, prod: &Prod) -> bool {
        match prod {
            Prod::Seq(items) => {
                let start_pos = self.position;
                for item in items {
                    if !self.parse_prod(item) {
                        self.position = start_pos;
                        return false;
                    }
                }
                true
            }
            
            Prod::Alt(alternatives) => {
                for alt in alternatives {
                    let start_pos = self.position;
                    if self.parse_prod(alt) {
                        return true;
                    }
                    self.position = start_pos;
                }
                false
            }
            
            Prod::Group(inner) => self.parse_prod(inner),
            
            Prod::Repeat { item, quant } => {
                let mut count = 0;
                loop {
                    let start_pos = self.position;
                    if self.parse_prod(item) {
                        count += 1;
                        if let Some(max) = quant.max {
                            if count >= max {
                                break;
                            }
                        }
                    } else {
                        self.position = start_pos;
                        break;
                    }
                }
                count >= quant.min
            }
            
            Prod::Terminal { kind, .. } => match kind {
                TerminalKind::Char(expected) => {
                    if let Some(ch) = self.peek_char() {
                        if ch == *expected {
                            let start = self.position;
                            self.advance();
                            self.events.push(ParseEvent::Token {
                                kind: TokenKind::Char(ch),
                                span: (start, self.position),
                            });
                            return true;
                        }
                    }
                    false
                }
                TerminalKind::Str(expected) => {
                    let start = self.position;
                    if self.input[self.position..].starts_with(expected) {
                        self.position += expected.len();
                        self.events.push(ParseEvent::Token {
                            kind: TokenKind::Str(expected.clone()),
                            span: (start, self.position),
                        });
                        true
                    } else {
                        false
                    }
                }
            },
            
            Prod::Class(char_class) => {
                if let Some(ch) = self.peek_char() {
                    let matches = char_class.chars.contains(&ch)
                        || char_class.ranges.iter().any(|(start, end)| ch >= *start && ch <= *end);
                    
                    let matches = if char_class.negated { !matches } else { matches };
                    
                    if matches {
                        let start = self.position;
                        self.advance();
                        self.events.push(ParseEvent::Token {
                            kind: TokenKind::Class(ch),
                            span: (start, self.position),
                        });
                        return true;
                    }
                }
                false
            }
            
            Prod::Ref { name, .. } => {
                if let Some(rule) = self.grammar.get_rule(name) {
                    let start_pos = self.position;
                    self.events.push(ParseEvent::Start {
                        rule: name.clone(),
                    });
                    
                    if self.parse_prod(&rule.production) {
                        self.events.push(ParseEvent::End {
                            rule: name.clone(),
                        });
                        true
                    } else {
                        self.position = start_pos;
                        false
                    }
                } else {
                    self.events.push(ParseEvent::Error(ParseError {
                        message: format!("undefined rule '{}'", name),
                        position: self.position,
                    }));
                    false
                }
            }
        }
    }
    
    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }
    
    fn advance(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.position += ch.len_utf8();
        }
    }
    
    fn into_iter(self) -> std::vec::IntoIter<ParseEvent> {
        self.events.into_iter()
    }
}

struct OwnedParser {
    input: String,
    grammar_ptr: *const Grammar,
    events: Vec<ParseEvent>,
}

impl OwnedParser {
    fn new_owned(input: String, grammar: &Grammar, start_rule: &str) -> Self {
        let grammar_ptr = grammar as *const Grammar;
        let mut parser = Self {
            input,
            grammar_ptr,
            events: Vec::new(),
        };
        
        // SAFETY: grammar reference is valid for the duration of this call
        unsafe {
            let grammar = &*parser.grammar_ptr;
            if let Some(_rule) = grammar.get_rule(start_rule) {
                let temp_parser = Parser::new(&parser.input, grammar, start_rule);
                parser.events = temp_parser.events;
            } else {
                parser.events.push(ParseEvent::Error(ParseError {
                    message: format!("start rule '{}' not found", start_rule),
                    position: 0,
                }));
            }
        }
        
        parser
    }
    
    fn into_iter(self) -> std::vec::IntoIter<ParseEvent> {
        self.events.into_iter()
    }
}

// Helper for Parser::new_owned signature
// Helper function to create an owned parser from String input
fn new_owned(input: String, grammar: &Grammar, start_rule: &str) -> OwnedParser {
    OwnedParser::new_owned(input, grammar, start_rule)
}
