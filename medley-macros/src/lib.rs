//! Procedural macros for the medley crate.
//!
//! This crate provides compile-time parsing for EBNF grammars without third-party dependencies.

use proc_macro::{Delimiter, Span, TokenStream, TokenTree};
use std::iter::Peekable;

/// Parse EBNF grammar in token-tree form at compile time.
///
/// # Example
///
/// ```ignore
/// let g = grammar! {
///     expr = term (('+' | '-') term)*;
///     term = number;
///     number = [0-9]+;
/// };
/// ```
#[proc_macro]
pub fn grammar(input: TokenStream) -> TokenStream {
    match parse_grammar(input) {
        Ok(code) => code,
        Err(e) => generate_error(e),
    }
}

#[derive(Debug)]
struct ParseError {
    message: String,
    span: Span,
}

type Result<T> = std::result::Result<T, ParseError>;

fn parse_grammar(input: TokenStream) -> Result<TokenStream> {
    let mut parser = Parser::new(input);
    let rules = parser.parse_rules()?;
    
    if rules.is_empty() {
        return Err(ParseError {
            message: "grammar has no rules".into(),
            span: Span::call_site(),
        });
    }
    
    Ok(generate_grammar_construction(rules))
}

struct Parser {
    tokens: Peekable<std::vec::IntoIter<TokenTree>>,
}

impl Parser {
    fn new(input: TokenStream) -> Self {
        Self {
            tokens: input.into_iter().collect::<Vec<_>>().into_iter().peekable(),
        }
    }
    
    fn peek(&mut self) -> Option<&TokenTree> {
        self.tokens.peek()
    }
    
    fn next(&mut self) -> Option<TokenTree> {
        self.tokens.next()
    }
    
    fn parse_rules(&mut self) -> Result<Vec<RuleAst>> {
        let mut rules = Vec::new();
        
        while self.peek().is_some() {
            rules.push(self.parse_rule()?);
        }
        
        Ok(rules)
    }
    
    fn parse_rule(&mut self) -> Result<RuleAst> {
        // name = production ;
        let name_token = self.next().ok_or_else(|| ParseError {
            message: "expected rule name".into(),
            span: Span::call_site(),
        })?;
        
        let name = match name_token {
            TokenTree::Ident(ident) => ident.to_string(),
            _ => return Err(ParseError {
                message: "expected identifier for rule name".into(),
                span: span_of(&name_token),
            }),
        };
        
        self.expect_punct('=')?;
        let production = self.parse_production()?;
        self.expect_punct(';')?;
        
        Ok(RuleAst { name, production })
    }
    
    fn parse_production(&mut self) -> Result<ProdAst> {
        let mut alternatives = vec![self.parse_sequence()?];
        
        while matches!(self.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '|') {
            self.next(); // consume |
            alternatives.push(self.parse_sequence()?);
        }
        
        if alternatives.len() == 1 {
            Ok(alternatives.into_iter().next().unwrap())
        } else {
            Ok(ProdAst::Alt(alternatives))
        }
    }
    
    fn parse_sequence(&mut self) -> Result<ProdAst> {
        let mut items = Vec::new();
        
        loop {
            match self.peek() {
                Some(TokenTree::Punct(p)) if p.as_char() == ';' || p.as_char() == '|' || p.as_char() == ')' => break,
                None => break,
                _ => items.push(self.parse_item()?),
            }
        }
        
        if items.is_empty() {
            return Err(ParseError {
                message: "expected at least one item in sequence".into(),
                span: Span::call_site(),
            });
        }
        
        if items.len() == 1 {
            Ok(items.into_iter().next().unwrap())
        } else {
            Ok(ProdAst::Seq(items))
        }
    }
    
    fn parse_item(&mut self) -> Result<ProdAst> {
        let mut item = self.parse_atom()?;
        
        // Check for repetition
        if let Some(TokenTree::Punct(p)) = self.peek() {
            let quant = match p.as_char() {
                '*' => Some((0, None)),
                '+' => Some((1, None)),
                '?' => Some((0, Some(1))),
                _ => None,
            };
            
            if let Some((min, max)) = quant {
                self.next(); // consume repetition operator
                item = ProdAst::Repeat {
                    item: Box::new(item),
                    min,
                    max,
                };
            }
        }
        
        Ok(item)
    }
    
    fn parse_atom(&mut self) -> Result<ProdAst> {
        let token = self.next().ok_or_else(|| ParseError {
            message: "unexpected end of input".into(),
            span: Span::call_site(),
        })?;
        
        match token {
            TokenTree::Literal(lit) => {
                let lit_str = lit.to_string();
                if lit_str.starts_with('"') && lit_str.ends_with('"') {
                    // String literal
                    let content = lit_str[1..lit_str.len() - 1].to_string();
                    Ok(ProdAst::StrLit(content))
                } else if lit_str.starts_with('\'') && lit_str.ends_with('\'') {
                    // Char literal
                    let content = lit_str[1..lit_str.len() - 1].to_string();
                    if content.len() == 1 {
                        Ok(ProdAst::CharLit(content.chars().next().unwrap()))
                    } else {
                        Err(ParseError {
                            message: "character literal must contain exactly one character".into(),
                            span: lit.span(),
                        })
                    }
                } else {
                    Err(ParseError {
                        message: "unexpected literal type".into(),
                        span: lit.span(),
                    })
                }
            }
            
            TokenTree::Ident(ident) => Ok(ProdAst::Ref(ident.to_string())),
            
            TokenTree::Group(group) => {
                match group.delimiter() {
                    Delimiter::Parenthesis => {
                        // Grouping
                        let mut inner_parser = Parser::new(group.stream());
                        let prod = inner_parser.parse_production()?;
                        Ok(ProdAst::Group(Box::new(prod)))
                    }
                    Delimiter::Bracket => {
                        // Character class
                        parse_char_class(group.stream())
                    }
                    _ => Err(ParseError {
                        message: "unexpected delimiter".into(),
                        span: group.span(),
                    }),
                }
            }
            
            TokenTree::Punct(p) => Err(ParseError {
                message: format!("unexpected punctuation '{}'", p.as_char()),
                span: p.span(),
            }),
        }
    }
    
    fn expect_punct(&mut self, expected: char) -> Result<()> {
        match self.next() {
            Some(TokenTree::Punct(p)) if p.as_char() == expected => Ok(()),
            Some(other) => Err(ParseError {
                message: format!("expected '{}', found {:?}", expected, other),
                span: span_of(&other),
            }),
            None => Err(ParseError {
                message: format!("expected '{}', found end of input", expected),
                span: Span::call_site(),
            }),
        }
    }
}

fn parse_char_class(stream: TokenStream) -> Result<ProdAst> {
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    
    // Check for negation
    let (negated, start_idx) = if let Some(TokenTree::Punct(p)) = tokens.first() {
        if p.as_char() == '^' {
            (true, 1)
        } else {
            (false, 0)
        }
    } else {
        (false, 0)
    };
    
    let mut chars = Vec::new();
    let mut ranges = Vec::new();
    let mut i = start_idx;
    
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Ident(ident) => {
                let s = ident.to_string();
                for ch in s.chars() {
                    chars.push(ch);
                }
                i += 1;
            }
            TokenTree::Literal(lit) => {
                let lit_str = lit.to_string();
                if lit_str.starts_with('\'') && lit_str.ends_with('\'') {
                    let ch = lit_str.chars().nth(1).unwrap();
                    
                    // Check if next is a dash (range)
                    if i + 2 < tokens.len() {
                        if let TokenTree::Punct(p) = &tokens[i + 1] {
                            if p.as_char() == '-' {
                                if let TokenTree::Literal(end_lit) = &tokens[i + 2] {
                                    let end_str = end_lit.to_string();
                                    if end_str.starts_with('\'') && end_str.ends_with('\'') {
                                        let end_ch = end_str.chars().nth(1).unwrap();
                                        ranges.push((ch, end_ch));
                                        i += 3;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    
                    chars.push(ch);
                } else {
                    for ch in lit_str.chars() {
                        chars.push(ch);
                    }
                }
                i += 1;
            }
            TokenTree::Punct(p) => {
                chars.push(p.as_char());
                i += 1;
            }
            TokenTree::Group(_) => {
                i += 1;
            }
        }
    }
    
    Ok(ProdAst::CharClass { negated, chars, ranges })
}

fn span_of(token: &TokenTree) -> Span {
    match token {
        TokenTree::Group(g) => g.span(),
        TokenTree::Ident(i) => i.span(),
        TokenTree::Punct(p) => p.span(),
        TokenTree::Literal(l) => l.span(),
    }
}

// AST types
#[derive(Debug)]
struct RuleAst {
    name: String,
    production: ProdAst,
}

#[derive(Debug)]
enum ProdAst {
    Seq(Vec<ProdAst>),
    Alt(Vec<ProdAst>),
    Group(Box<ProdAst>),
    Repeat { item: Box<ProdAst>, min: usize, max: Option<usize> },
    CharLit(char),
    StrLit(String),
    CharClass { negated: bool, chars: Vec<char>, ranges: Vec<(char, char)> },
    Ref(String),
}

fn generate_grammar_construction(rules: Vec<RuleAst>) -> TokenStream {
    let mut code = String::new();
    
    code.push_str("{\n");
    code.push_str("    use ::medley::ebnf::*;\n");
    code.push_str("    Grammar {\n");
    code.push_str("        rules: vec![\n");
    
    for rule in rules {
        code.push_str(&format!("            Rule {{\n"));
        code.push_str(&format!("                name: {:?}.to_string(),\n", rule.name));
        code.push_str(&format!("                production: {},\n", generate_prod(&rule.production)));
        code.push_str(&format!("                span: None,\n"));
        code.push_str(&format!("            }},\n"));
    }
    
    code.push_str("        ],\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    
    code.parse().unwrap()
}

fn generate_prod(prod: &ProdAst) -> String {
    match prod {
        ProdAst::Seq(items) => {
            let items_code: Vec<String> = items.iter().map(generate_prod).collect();
            format!("Prod::Seq(vec![{}])", items_code.join(", "))
        }
        ProdAst::Alt(items) => {
            let items_code: Vec<String> = items.iter().map(generate_prod).collect();
            format!("Prod::Alt(vec![{}])", items_code.join(", "))
        }
        ProdAst::Group(inner) => {
            format!("Prod::Group(Box::new({}))", generate_prod(inner))
        }
        ProdAst::Repeat { item, min, max } => {
            let max_str = match max {
                Some(n) => format!("Some({})", n),
                None => "None".to_string(),
            };
            format!(
                "Prod::Repeat {{ item: Box::new({}), quant: RepeatQuant {{ min: {}, max: {} }} }}",
                generate_prod(item),
                min,
                max_str
            )
        }
        ProdAst::CharLit(ch) => {
            format!("Prod::Terminal {{ kind: TerminalKind::Char({:?}), span: None }}", ch)
        }
        ProdAst::StrLit(s) => {
            format!("Prod::Terminal {{ kind: TerminalKind::Str({:?}.to_string()), span: None }}", s)
        }
        ProdAst::CharClass { negated, chars, ranges } => {
            let chars_code = format!("vec![{}]", chars.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>().join(", "));
            let ranges_code = format!("vec![{}]", ranges.iter().map(|(a, b)| format!("({:?}, {:?})", a, b)).collect::<Vec<_>>().join(", "));
            format!(
                "Prod::Class(CharClass {{ negated: {}, chars: {}, ranges: {}, span: None }})",
                negated, chars_code, ranges_code
            )
        }
        ProdAst::Ref(name) => {
            format!("Prod::Ref {{ name: {:?}.to_string(), span: None }}", name)
        }
    }
}

fn generate_error(err: ParseError) -> TokenStream {
    // Use proc_macro::Literal to attach the span to the error
    let message = err.message;
    let span = err.span;
    
    // Create a compile_error! invocation with the proper span
    let error_msg = proc_macro::Literal::string(&message);
    let mut error_lit = TokenTree::Literal(error_msg);
    error_lit.set_span(span);
    
    let compile_error = TokenTree::Ident(proc_macro::Ident::new("compile_error", span));
    let punct_bang = TokenTree::Punct(proc_macro::Punct::new('!', proc_macro::Spacing::Alone));
    let group = TokenTree::Group(proc_macro::Group::new(
        Delimiter::Parenthesis,
        TokenStream::from(error_lit)
    ));
    
    TokenStream::from_iter(vec![compile_error, punct_bang, group])
}
