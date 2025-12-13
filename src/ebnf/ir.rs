//! Grammar Internal Representation (IR) types

/// A byte-span within the grammar source (for diagnostics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Terminal literal kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalKind {
    /// Single character literal, e.g., 'a'
    Char(char),
    /// String literal, e.g., "if"
    Str(String),
}

/// Character class specification (basic ASCII for Phase 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharClass {
    pub negated: bool,
    /// Individual allowed characters
    pub chars: Vec<char>,
    /// Inclusive ranges (start..=end)
    pub ranges: Vec<(char, char)>,
    pub span: Option<Span>,
}

/// Repetition quantifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatQuant {
    pub min: usize,
    pub max: Option<usize>,
}

/// A production node in the grammar IR.
#[derive(Debug, Clone, PartialEq)]
pub enum Prod {
    /// Sequence of items: a b c
    Seq(Vec<Prod>),
    /// Alternation: a | b | c
    Alt(Vec<Prod>),
    /// Grouped sub-production: ( ... )
    Group(Box<Prod>),
    /// Repetition of an item with quantifier (*, +, ? maps to specific ranges)
    Repeat { item: Box<Prod>, quant: RepeatQuant },
    /// Terminal literal
    Terminal { kind: TerminalKind, span: Option<Span> },
    /// Character class
    Class(CharClass),
    /// Reference to another rule by name
    Ref { name: String, span: Option<Span> },
}

/// A rule definition: name = production;
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    pub production: Prod,
    pub span: Option<Span>,
}

/// A full grammar.
#[derive(Debug, Clone, PartialEq)]
pub struct Grammar {
    pub rules: Vec<Rule>,
    /// Optional name of the start rule; defaults to the first rule if None.
    pub start: Option<String>,
}

impl Grammar {
    /// Return the rule by name, if present.
    pub fn get_rule(&self, name: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.name == name)
    }

    /// Basic validation: undefined references, empty rules, and start rule existence.
    /// Phase 2 keeps this minimal; deeper checks will be added in Phase 5.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.rules.is_empty() {
            errors.push("grammar has no rules".to_string());
            return errors;
        }

        // Determine start rule
        let start_name = self
            .start
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|| self.rules[0].name.as_str());
        if self.get_rule(start_name).is_none() {
            errors.push(format!("start rule '{}' not found", start_name));
        }

        // Collect defined rule names
        let defined: std::collections::HashSet<&str> =
            self.rules.iter().map(|r| r.name.as_str()).collect();

        // Walk productions and check references
        fn check_prod(p: &Prod, defined: &std::collections::HashSet<&str>, errors: &mut Vec<String>) {
            match p {
                Prod::Seq(items) | Prod::Alt(items) => {
                    for it in items {
                        check_prod(it, defined, errors);
                    }
                }
                Prod::Group(inner) => check_prod(inner, defined, errors),
                Prod::Repeat { item, .. } => check_prod(item, defined, errors),
                Prod::Terminal { .. } => {}
                Prod::Class(_) => {}
                Prod::Ref { name, .. } => {
                    if !defined.contains(name.as_str()) {
                        errors.push(format!("undefined rule '{}'", name));
                    }
                }
            }
        }

        for rule in &self.rules {
            // Empty production check (Seq([]) etc.) is simplistic here
            check_prod(&rule.production, &defined, &mut errors);
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(name: &str, production: Prod) -> Rule {
        Rule { name: name.to_string(), production, span: None }
    }

    #[test]
    fn valid_simple_grammar_has_no_errors_and_infers_start() {
        let g = Grammar {
            rules: vec![
                rule(
                    "digit",
                    Prod::Class(CharClass { negated: false, chars: vec![], ranges: vec![("0", "9")].into_iter().map(|(a,b)| (a.chars().next().unwrap(), b.chars().next().unwrap())).collect(), span: None }),
                ),
                rule(
                    "number",
                    Prod::Repeat { item: Box::new(Prod::Ref { name: "digit".into(), span: None }), quant: RepeatQuant { min: 1, max: None } },
                ),
            ],
            start: None,
        };

        let errors = g.validate();
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
        assert!(g.get_rule("digit").is_some());
    }

    #[test]
    fn undefined_rule_reference_is_reported() {
        let g = Grammar {
            rules: vec![rule("start", Prod::Ref { name: "missing".into(), span: None })],
            start: Some("start".into()),
        };
        let errors = g.validate();
        assert!(errors.iter().any(|e| e.contains("undefined rule 'missing'")), "errors: {:?}", errors);
    }

    #[test]
    fn invalid_start_rule_is_reported() {
        let g = Grammar { rules: vec![rule("a", Prod::Terminal { kind: TerminalKind::Char('x'), span: None })], start: Some("nope".into()) };
        let errors = g.validate();
        assert!(errors.iter().any(|e| e.contains("start rule 'nope' not found")), "errors: {:?}", errors);
    }
}
