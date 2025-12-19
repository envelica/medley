# EBNF Grammar Specification for `grammar!` Macro

## Overview

The `grammar!` macro accepts token-tree form only and allows developers to embed EBNF grammar definitions directly in Rust source code. This document specifies the syntax, semantics, constraints, and usage patterns.

## Syntax

### Grammar Structure

A grammar is a collection of rules separated by newlines or whitespace:

```rust
let my_grammar = grammar! {
    rule1 = production1;
    rule2 = production2;
    rule3 = production3;
};
```

### Rule Definition

```
rule_name = production ;
```

- **rule_name**: An identifier (alphanumeric + underscore) that names a production.
- **production**: A sequence of operators and terminals defined below.
- **`;`**: Semicolon terminates the rule.

Rules are case-sensitive. Convention: lowercase for rules (e.g., `expr`, `term`), UPPERCASE for token kinds (e.g., `DIGIT`, `ALPHA`).

### Productions

A production is built from the following primitives:

#### 1. Terminals (Literal Strings)

Quoted strings match literal characters or sequences:

```rust
// Single-quoted: matches exactly one character
'a'    // matches 'a'
'('    // matches '('

// Double-quoted: matches the full string
"if"   // matches "if"
"else" // matches "else"
```

Escaping:
- `\'` for single quote in single-quoted terminal
- `\"` for double quote in double-quoted terminal
- `\\` for backslash
- `\n`, `\t`, `\r` for common whitespace (future: review scope)

#### 2. Character Classes and Ranges

Define a set of characters using brackets:

```rust
[a-z]       // matches any lowercase letter
[A-Z]       // matches any uppercase letter
[0-9]       // matches any digit
[a-zA-Z0-9] // matches alphanumeric
[^a-z]      // matches any character NOT in a-z (negation with ^)
[a\-z]      // matches 'a', '-', 'z' (escaped dash to include literal -)
```

Inside `[ ]`:
- Ranges: `a-z`, `0-9`, `A-Z`
- Individual characters: `a`, `1`, `_`
- Negation: `^` at the start, e.g., `[^abc]`
- Escape sequences: `\n`, `\t`, `\'`, `\\`, etc.

#### 3. Rule References

Reference a previously or subsequently defined rule:

```rust
identifier = letter (letter | digit)*;
letter = [a-z] | [A-Z];
digit = [0-9];
```

Forward references are allowed (as long as no cycles exist, or cycles are acceptable for left-recursive handling TBD).

#### 4. Operators

##### Concatenation (Implicit)

Adjacent items match in sequence:

```rust
digit letter digit  // matches: digit, then letter, then digit
```

##### Alternation (`|`)

Match one of several options:

```rust
sign = '+' | '-';
```

Priority: alternation binds loosely; grouping applies tight precedence.

##### Repetition

- `*` (zero or more)
- `+` (one or more)
- `?` (zero or one, i.e., optional)

```rust
digits = digit+;           // one or more digits
optional_sign = sign?;     // optional sign
spaces = ' '+;             // one or more spaces
whitespace = [ \t\n]*;     // zero or more whitespace
```

##### Grouping

Parentheses override precedence:

```rust
expr = term (('+' | '-') term)*;
```

Without grouping:
```rust
// Ambiguous: does this mean (term +) or term (+ term)*?
expr = term + '-' term *;
```

##### Optional (Future Consideration)

Brackets `[ ]` for optional groups (if included in subset):

```rust
optional_expr = [expr];
```

**Note**: Currently using brackets for character classes. Optional groups may require different syntax or context awareness.

## Operator Precedence (Highest to Lowest)

1. Terminals, character classes, rule references, parentheses/grouping
2. Repetition (`*`, `+`, `?`)
3. Concatenation
4. Alternation (`|`)

Examples:

```rust
// Parses as: a (b c)
rule1 = a b c;

// Parses as: a (b | c)
rule2 = a b | c;  // AMBIGUOUS: clarify as (a b) | c or a (b | c)?

// To be explicit:
rule3 = (a b) | c;      // alternation of two sequences
rule4 = a (b | c);      // concatenation: a followed by (b or c)
```

**Clarification Note**: If `a b | c` is ambiguous, require explicit parentheses:
- Use `(a b) | c` for alternation.
- Use `a (b | c)` for alternation within a sequence.

## Whitespace and Comments

- Whitespace (spaces, tabs, newlines) is ignored outside of terminals.
- Comments: **Not in Phase 1; defer to Phase 2+ if needed.**
- Inside terminals, whitespace is literal.

Example:

```rust
// These are equivalent:
rule1 = 'a' 'b' 'c';
rule2 = 'a'
         'b'
         'c';
```

## Unsupported Constructs (Phase 1)

The following are NOT supported in Phase 1; they may be added in future phases:

1. **Left Recursion**: Rules like `rule = rule 'x' | 'y';` are undefined behavior. Detection and error messages TBD in Phase 5 (Diagnostics).
2. **Semantic Actions**: No code blocks, transformations, or AST construction directives.
3. **Lookahead**: No explicit lookahead operators.
4. **Named Captures**: No named groups or subtree extraction (covered in Phase 6+ via AST builder).
5. **Comments**: Deferred.
6. **Unicode Escapes**: Currently only ASCII; full Unicode support deferred.
7. **Whitespace Suppression**: No automatic whitespace skipping; must be explicit in rules.

## Complete Example

```rust
let arithmetic = grammar! {
    expr   = term (('+' | '-') term)*;
    term   = factor (('*' | '/') factor)*;
    factor = number | '(' expr ')';
    number = digit+;
    digit  = [0-9];
};
```

## Macro Invocation

The `grammar!` macro is invoked in token-tree form:

```rust
let g = grammar! {
    rule1 = production1;
    rule2 = production2;
};
```

## Compile-Time vs. Runtime Validation

- **Compile-time (best-effort in Phase 3)**:
  - Basic syntax errors (malformed rules, unexpected tokens).
  - Simple well-formedness checks (closing braces match, etc.).
  - Diagnostics will be clear compile errors where possible.

- **Runtime (fallback in Phase 3, enhanced in Phase 5)**:
  - Undefined rule references.
  - Cyclic/recursive rule detection.
  - Other semantic checks.

## Error Messages

Clear, actionable errors:

```rust
error[E0001]: undefined rule 'undefined_rule' in grammar
 --> src/main.rs:5:20
  |
5 |     rule = undefined_rule;
  |            ^^^^^^^^^^^^^^
  |
  = hint: check rule name or define it above
```

## Future Extensions (Out of Scope for Phase 1)

- Comments within rules.
- Negative lookahead / positive lookahead.
- Semantic actions (code execution on match).
- Lexer generation or token definitions.
- Grammar inheritance or composition.
