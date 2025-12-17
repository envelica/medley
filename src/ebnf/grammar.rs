//! Grammar Internal Representation (IR) types

use super::Span;

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
}

impl Grammar {
    /// Return the rule by name, if present.
    pub fn get_rule(&self, name: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.name == name)
    }

    /// Return the start rule (always the first rule).
    pub fn start_rule(&self) -> Option<&Rule> {
        self.rules.first()
    }

    /// Basic validation: undefined references, empty rules, and start rule existence.
    /// Phase 5: Includes left-recursion detection.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.rules.is_empty() {
            errors.push("grammar has no rules".to_string());
            return errors;
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

        // Check for left recursion
        self.check_left_recursion(&mut errors);

        // Check for cyclic dependencies
        self.check_cyclic_dependencies(&mut errors);

        errors
    }

    /// Detect direct and indirect left recursion.
    fn check_left_recursion(&self, errors: &mut Vec<String>) {
        use std::collections::{HashMap, HashSet};

        // Build rule lookup map
        let rule_map: HashMap<&str, &Prod> = self.rules.iter()
            .map(|r| (r.name.as_str(), &r.production))
            .collect();

        // Check each rule for left recursion
        for rule in &self.rules {
            let mut visited = HashSet::new();
            let mut path = Vec::new();
            if self.is_left_recursive(&rule.name, &rule_map, &mut visited, &mut path) {
                let cycle = path.join(" -> ");
                errors.push(format!("left recursion detected: {}", cycle));
            }
        }
    }

    /// Check if a rule is left-recursive by examining its leftmost derivations.
    fn is_left_recursive(
        &self,
        rule_name: &str,
        rule_map: &std::collections::HashMap<&str, &Prod>,
        visited: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        // Add to path
        path.push(rule_name.to_string());

        // If we've seen this rule in the current path, we have a cycle
        if visited.contains(rule_name) {
            return true;
        }

        visited.insert(rule_name.to_string());

        // Get the production for this rule
        let prod = match rule_map.get(rule_name) {
            Some(p) => p,
            None => {
                path.pop();
                visited.remove(rule_name);
                return false;
            }
        };

        // Check if any leftmost symbol can lead back to this rule
        let result = self.check_prod_left_recursion(prod, rule_map, visited, path);

        // Backtrack
        visited.remove(rule_name);
        path.pop();

        result
    }

    /// Check if a production's leftmost symbols can lead to left recursion.
    fn check_prod_left_recursion(
        &self,
        prod: &Prod,
        rule_map: &std::collections::HashMap<&str, &Prod>,
        visited: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        match prod {
            Prod::Seq(items) => {
                // Check the first item in sequence (leftmost)
                if let Some(first) = items.first() {
                    self.check_prod_left_recursion(first, rule_map, visited, path)
                } else {
                    false
                }
            }
            Prod::Alt(alts) => {
                // Check if any alternative is left-recursive
                alts.iter().any(|alt| self.check_prod_left_recursion(alt, rule_map, visited, path))
            }
            Prod::Group(inner) => {
                self.check_prod_left_recursion(inner, rule_map, visited, path)
            }
            Prod::Repeat { item, quant } => {
                // Repeat with min=0 is nullable, so check the item
                // Repeat with min>0 checks the item
                if quant.min == 0 {
                    false // Nullable, doesn't contribute to left recursion
                } else {
                    self.check_prod_left_recursion(item, rule_map, visited, path)
                }
            }
            Prod::Terminal { .. } | Prod::Class(_) => {
                false // Terminals break left recursion
            }
            Prod::Ref { name, .. } => {
                // This is a reference; check if it leads back to a rule in our path
                self.is_left_recursive(name, rule_map, visited, path)
            }
        }
    }

    /// Detect cyclic dependencies (only problematic ones - pure reference cycles without terminals).
    /// Expression grammars with mutual recursion but terminals are valid and not flagged.
    fn check_cyclic_dependencies(&self, errors: &mut Vec<String>) {
        use std::collections::{HashMap, HashSet};

        // Build rule lookup map
        let rule_map: HashMap<&str, &Prod> = self.rules.iter()
            .map(|r| (r.name.as_str(), &r.production))
            .collect();

        // Track which rules we've fully processed
        let mut fully_processed = HashSet::new();

        // Check each rule for problematic cycles (pure reference cycles)
        for rule in &self.rules {
            if fully_processed.contains(rule.name.as_str()) {
                continue;
            }

            let mut visiting = HashSet::new();
            let mut path = Vec::new();
            if self.has_problematic_cycle(&rule.name, &rule_map, &mut visiting, &mut fully_processed, &mut path) {
                let cycle = path.join(" -> ");
                errors.push(format!("cyclic dependency detected: {}", cycle));
            }
        }
    }

    /// Check if a rule has a problematic cyclic dependency (pure reference cycle without terminals).
    fn has_problematic_cycle(
        &self,
        rule_name: &str,
        rule_map: &std::collections::HashMap<&str, &Prod>,
        visiting: &mut std::collections::HashSet<String>,
        fully_processed: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        // If already fully processed, no cycle from this node
        if fully_processed.contains(rule_name) {
            return false;
        }

        // If currently visiting, check if this is a pure reference cycle
        if visiting.contains(rule_name) {
            // Only report if there are no terminals in the cycle path
            // (Expression grammars with terminals are valid)
            path.push(rule_name.to_string());
            return self.is_pure_reference_cycle(rule_name, rule_map, &visiting);
        }

        path.push(rule_name.to_string());
        visiting.insert(rule_name.to_string());

        let prod = match rule_map.get(rule_name) {
            Some(p) => p,
            None => {
                visiting.remove(rule_name);
                path.pop();
                return false;
            }
        };

        // Collect all rule references in this production
        let mut refs = Vec::new();
        self.collect_rule_refs(prod, &mut refs);

        // Check each referenced rule
        for ref_name in refs {
            if self.has_problematic_cycle(&ref_name, rule_map, visiting, fully_processed, path) {
                return true;
            }
        }

        // Mark as fully processed
        visiting.remove(rule_name);
        fully_processed.insert(rule_name.to_string());
        path.pop();
        false
    }

    /// Check if all rules in a cycle contain only references (no terminals).
    fn is_pure_reference_cycle(
        &self,
        _rule_name: &str,
        rule_map: &std::collections::HashMap<&str, &Prod>,
        visiting: &std::collections::HashSet<String>,
    ) -> bool {
        for name in visiting {
            if let Some(prod) = rule_map.get(name.as_str()) {
                if self.prod_has_terminal(prod) {
                    return false; // Has a terminal, not a pure reference cycle
                }
            }
        }
        true
    }

    /// Check if a production contains any terminals.
    fn prod_has_terminal(&self, prod: &Prod) -> bool {
        match prod {
            Prod::Seq(items) | Prod::Alt(items) => {
                items.iter().any(|item| self.prod_has_terminal(item))
            }
            Prod::Group(inner) => self.prod_has_terminal(inner),
            Prod::Repeat { item, .. } => self.prod_has_terminal(item),
            Prod::Terminal { .. } | Prod::Class(_) => true,
            Prod::Ref { .. } => false,
        }
    }

    /// Collect all rule references from a production.
    fn collect_rule_refs(&self, prod: &Prod, refs: &mut Vec<String>) {
        match prod {
            Prod::Seq(items) | Prod::Alt(items) => {
                for item in items {
                    self.collect_rule_refs(item, refs);
                }
            }
            Prod::Group(inner) => {
                self.collect_rule_refs(inner, refs);
            }
            Prod::Repeat { item, .. } => {
                self.collect_rule_refs(item, refs);
            }
            Prod::Terminal { .. } | Prod::Class(_) => {}
            Prod::Ref { name, .. } => {
                refs.push(name.clone());
            }
        }
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
        };

        let errors = g.validate();
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
        assert!(g.get_rule("digit").is_some());
    }

    #[test]
    fn undefined_rule_reference_is_reported() {
        let g = Grammar {
            rules: vec![rule("start", Prod::Ref { name: "missing".into(), span: None })],
        };
        let errors = g.validate();
        assert!(errors.iter().any(|e| e.contains("undefined rule 'missing'")), "errors: {:?}", errors);
    }

    #[test]
    fn direct_left_recursion_is_detected() {
        // expr = expr '+' term | term
        let g = Grammar {
            rules: vec![
                rule(
                    "expr",
                    Prod::Alt(vec![
                        Prod::Seq(vec![
                            Prod::Ref { name: "expr".into(), span: None },
                            Prod::Terminal { kind: TerminalKind::Char('+'), span: None },
                            Prod::Ref { name: "term".into(), span: None },
                        ]),
                        Prod::Ref { name: "term".into(), span: None },
                    ]),
                ),
                rule("term", Prod::Terminal { kind: TerminalKind::Char('x'), span: None }),
            ],
        };
        let errors = g.validate();
        assert!(errors.iter().any(|e| e.contains("left recursion") && e.contains("expr")), "errors: {:?}", errors);
    }

    #[test]
    fn indirect_left_recursion_is_detected() {
        // a = b; b = a
        let g = Grammar {
            rules: vec![
                rule("a", Prod::Ref { name: "b".into(), span: None }),
                rule("b", Prod::Ref { name: "a".into(), span: None }),
            ],
        };
        let errors = g.validate();
        assert!(errors.iter().any(|e| e.contains("left recursion")), "errors: {:?}", errors);
    }

    #[test]
    fn right_recursion_is_not_flagged() {
        // expr = term '+' expr | term
        let g = Grammar {
            rules: vec![
                rule(
                    "expr",
                    Prod::Alt(vec![
                        Prod::Seq(vec![
                            Prod::Ref { name: "term".into(), span: None },
                            Prod::Terminal { kind: TerminalKind::Char('+'), span: None },
                            Prod::Ref { name: "expr".into(), span: None },
                        ]),
                        Prod::Ref { name: "term".into(), span: None },
                    ]),
                ),
                rule("term", Prod::Terminal { kind: TerminalKind::Char('x'), span: None }),
            ],
        };
        let errors = g.validate();
        assert!(!errors.iter().any(|e| e.contains("left recursion")), "errors: {:?}", errors);
    }

    #[test]
    fn pure_reference_cycle_is_detected() {
        // a = b; b = a  (pure reference cycle without terminals)
        let g = Grammar {
            rules: vec![
                rule("a", Prod::Ref { name: "b".into(), span: None }),
                rule("b", Prod::Ref { name: "a".into(), span: None }),
            ],
        };
        let errors = g.validate();
        assert!(errors.iter().any(|e| e.contains("cyclic dependency")), "errors: {:?}", errors);
    }

    #[test]
    fn indirect_pure_cycle_is_detected() {
        // a = b; b = c; c = a  (three-way pure reference cycle)
        let g = Grammar {
            rules: vec![
                rule("a", Prod::Ref { name: "b".into(), span: None }),
                rule("b", Prod::Ref { name: "c".into(), span: None }),
                rule("c", Prod::Ref { name: "a".into(), span: None }),
            ],
        };
        let errors = g.validate();
        assert!(errors.iter().any(|e| e.contains("cyclic dependency")), "errors: {:?}", errors);
    }

    #[test]
    fn cycle_with_terminals_is_not_flagged() {
        // a = 'x' a 'y' | 'z'  (cycle with terminals - valid, like expression grammars)
        let g = Grammar {
            rules: vec![
                rule(
                    "a",
                    Prod::Alt(vec![
                        Prod::Seq(vec![
                            Prod::Terminal { kind: TerminalKind::Char('x'), span: None },
                            Prod::Ref { name: "a".into(), span: None },
                            Prod::Terminal { kind: TerminalKind::Char('y'), span: None },
                        ]),
                        Prod::Terminal { kind: TerminalKind::Char('z'), span: None },
                    ]),
                ),
            ],
        };
        let errors = g.validate();
        assert!(!errors.iter().any(|e| e.contains("cyclic dependency")), "errors: {:?}", errors);
    }

    #[test]
    fn no_cycle_with_proper_termination() {
        // a = b | 'x'; b = c | 'y'; c = 'z'
        let g = Grammar {
            rules: vec![
                rule(
                    "a",
                    Prod::Alt(vec![
                        Prod::Ref { name: "b".into(), span: None },
                        Prod::Terminal { kind: TerminalKind::Char('x'), span: None },
                    ]),
                ),
                rule(
                    "b",
                    Prod::Alt(vec![
                        Prod::Ref { name: "c".into(), span: None },
                        Prod::Terminal { kind: TerminalKind::Char('y'), span: None },
                    ]),
                ),
                rule("c", Prod::Terminal { kind: TerminalKind::Char('z'), span: None }),
            ],
        };
        let errors = g.validate();
        assert!(!errors.iter().any(|e| e.contains("cyclic dependency")), "errors: {:?}", errors);
    }

}
