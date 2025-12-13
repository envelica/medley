//! Streaming, pull-based parser interface.
//! Designed to traverse input lazily from any `BufRead` and emit events one at a time.

use std::collections::VecDeque;
use std::io::{BufRead, Cursor};

use crate::ebnf::ir::{CharClass, Grammar, Prod, RepeatQuant, Span, TerminalKind};

/// Parse event emitted by the pull parser.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseEvent<'a> {
    /// Start of a rule match.
    Start { rule: &'a str },
    /// End of a rule match.
    End { rule: &'a str },
    /// Token matched (terminal or character class).
    Token { kind: TokenKind<'a>, span: Span },
    /// Parse error encountered.
    Error(ParseError),
}

/// Token kind for parsed terminals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind<'a> {
    Char(char),
    Str(&'a str),
    Class(char),
}

/// Enhanced parse error with diagnostic information.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
    pub span: Option<Span>,
    pub rule_context: Option<String>,
    pub hint: Option<String>,
}

impl ParseError {
    pub fn new(message: impl Into<String>, position: usize) -> Self {
        Self { message: message.into(), position, span: None, rule_context: None, hint: None }
    }
    pub fn with_span(mut self, span: Span) -> Self { self.span = Some(span); self }
    pub fn with_rule_context(mut self, rule: impl Into<String>) -> Self { self.rule_context = Some(rule.into()); self }
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self { self.hint = Some(hint.into()); self }
}

/// Tracks line and column numbers during parsing (internal only)
#[derive(Debug, Clone)]
struct LineColumnTracker {
    positions: Vec<usize>, // Byte positions of line starts
    input_len: usize,      // Track input length for extend
}

impl LineColumnTracker {
    fn new(input: &str) -> Self {
        let mut positions = vec![0];
        for (i, ch) in input.char_indices() {
            if ch == '\n' { positions.push(i + 1); }
        }
        Self { positions, input_len: input.len() }
    }
    fn line_column(&self, position: usize) -> (u32, u32) {
        // Binary search for the line containing this position
        let line = match self.positions.binary_search(&position) {
            Ok(idx) => idx + 1,  // Exact match at line start
            Err(idx) => idx,     // Position falls between line starts
        };
        let actual_line = if line == 0 { 1 } else { line as u32 };
        let line_start = if line == 0 { 0 } else { self.positions[line - 1] };
        let column = (position - line_start) as u32 + 1;
        (actual_line, column)
    }
    fn span_with_position(&self, start: usize, end: usize) -> Span {
        let (line, column) = self.line_column(start);
        Span::with_position(start, end, line, column)
    }
    fn extend(&mut self, chunk: &str) {
        let base = self.input_len;
        self.input_len += chunk.len();
        for (i, ch) in chunk.char_indices() {
            if ch == '\n' { self.positions.push(base + i + 1); }
        }
    }
}

/// Internal parse frame representing progress within a production.
#[derive(Clone, Debug)]
enum Frame<'a> {
    Seq { items: &'a [Prod], idx: usize },
    Alt { alts: &'a [Prod], idx: usize, saved_pos: usize },
    Group { inner: &'a Prod },
    Repeat { item: &'a Prod, quant: &'a RepeatQuant, count: usize, saved_pos: usize, trying: bool },
    Terminal { kind: &'a TerminalKind },
    Class { class: &'a CharClass },
    Ref { name: &'a str, prod: &'a Prod, stage: RefStage },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RefStage {
    Start,
    Parsing,
    End,
}

/// Streaming parser that emits `ParseEvent` lazily.
///
/// Uses a sliding window buffer to bound memory usage for large inputs.
/// Backtracking is limited to data within the current window.
pub struct Parser<'a, R: BufRead> {
    grammar: &'a Grammar,
    input: String,
    pos: usize,
    window_start: usize, // Absolute position where input buffer starts
    frames: Vec<Frame<'a>>,
    events: VecDeque<ParseEvent<'a>>,
    line_tracker: LineColumnTracker,
    finished: bool,
    reader: Option<R>,
    eof: bool,
}

impl<'a, R: BufRead> Parser<'a, R> {
    /// Create a new streaming parser over the given reader and grammar.
    pub fn new(grammar: &'a Grammar, reader: R) -> Self {
        let input = String::new();

        let line_tracker = LineColumnTracker::new("");
        let mut parser = Self {
            grammar,
            input,
            pos: 0,
            window_start: 0,
            frames: Vec::new(),
            events: VecDeque::new(),
            line_tracker,
            finished: false,
            reader: Some(reader),
            eof: false,
        };

        if let Some(rule) = grammar.rules.first() {
            parser.frames.push(Frame::Ref {
                name: rule.name.as_str(),
                prod: &rule.production,
                stage: RefStage::Start,
            });
        } else {
            parser.finished = true;
            parser.events.push_back(ParseEvent::Error(ParseError::new("grammar has no rules", 0)));
        }

        parser
    }
}

impl<'a> Parser<'a, Cursor<&'a [u8]>> {
    /// Create a parser from a string input.
    pub fn from_str(grammar: &'a Grammar, input: &'a str) -> Self {
        Self::new(grammar, Cursor::new(input.as_bytes()))
    }
}

impl<'a, R: BufRead> Parser<'a, R> {

    /// Advance and return the next parse event, or `None` when finished.
    pub fn next_event(&mut self) -> Option<ParseEvent<'a>> {
        if let Some(ev) = self.events.pop_front() {
            return Some(ev);
        }

        if self.finished {
            return None;
        }

        while !self.finished {
            if let Some(ev) = self.step() {
                return Some(ev);
            }
            if let Some(ev) = self.events.pop_front() {
                return Some(ev);
            }
        }

        self.events.pop_front()
    }

            fn step(&mut self) -> Option<ParseEvent<'a>> {
        let frame = match self.frames.pop() {
            Some(f) => f,
            None => {
                self.finished = true;
                return None;
            }
        };

        match frame {
            Frame::Ref { name, prod, stage } => match stage {
                RefStage::Start => {
                    self.events.push_back(ParseEvent::Start { rule: name });
                    self.frames.push(Frame::Ref { name, prod, stage: RefStage::Parsing });
                    self.frames.push(Frame::from_prod(prod));
                    None
                }
                RefStage::Parsing => {
                    // Child finished successfully
                    self.frames.push(Frame::Ref { name, prod, stage: RefStage::End });
                    None
                }
                RefStage::End => {
                    self.events.push_back(ParseEvent::End { rule: name });
                    None
                }
            },

            Frame::Seq { items, idx } => {
                if idx < items.len() {
                    self.frames.push(Frame::Seq { items, idx: idx + 1 });
                    self.frames.push(Frame::from_prod(&items[idx]));
                }
                None
            }

            Frame::Alt { alts, idx, saved_pos } => {
                if idx == 0 {
                    // First time seeing this Alt frame - save current position
                    let current_pos = self.pos;
                    self.frames.push(Frame::Alt { alts, idx: 1, saved_pos: current_pos });
                    self.frames.push(Frame::from_prod(&alts[0]));
                } else if idx < alts.len() {
                    // Alternative failed, restore position and try next
                    self.pos = saved_pos;
                    self.frames.push(Frame::Alt { alts, idx: idx + 1, saved_pos });
                    self.frames.push(Frame::from_prod(&alts[idx]));
                } else {
                    // All alternatives failed
                    self.fail("no alternative matched");
                }
                None
            }

            Frame::Group { inner } => {
                self.frames.push(Frame::from_prod(inner));
                None
            }

            Frame::Repeat { item, quant, count, saved_pos, trying } => {
                if trying {
                    if let Some(max) = quant.max {
                        if count >= max {
                            // Stop repeating
                            return None;
                        }
                    }
                    self.frames.push(Frame::Repeat { item, quant, count, saved_pos: self.pos, trying: false });
                    self.frames.push(Frame::from_prod(item));
                } else {
                    // After attempting one repetition
                    if self.pos > saved_pos {
                        // Made progress; try again
                        self.frames.push(Frame::Repeat { item, quant, count: count + 1, saved_pos: self.pos, trying: true });
                    } else {
                        // No progress; check min bound
                        if count < quant.min {
                            self.fail("repeat did not satisfy minimum occurrences");
                        }
                    }
                }
                None
            }

            Frame::Terminal { kind } => {
                let start = self.window_start + self.pos;
                let matched = match kind {
                    TerminalKind::Char(expected) => {
                        self.ensure_buffer(1);
                        if let Some((ch, len)) = self.peek_char() {
                            if ch == *expected {
                                self.consume(len);
                                self.events.push_back(ParseEvent::Token {
                                    kind: TokenKind::Char(ch),
                                    span: self.line_tracker.span_with_position(start, self.window_start + self.pos),
                                });
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    TerminalKind::Str(expected) => {
                        self.ensure_buffer(expected.len());
                        if self.input[self.pos..].starts_with(expected) {
                            let bytes = expected.len();
                            self.consume(bytes);
                            self.events.push_back(ParseEvent::Token {
                                kind: TokenKind::Str(expected),
                                span: self.line_tracker.span_with_position(start, self.window_start + self.pos),
                            });
                            true
                        } else {
                            false
                        }
                    }
                };

                if !matched {
                    self.fail("terminal did not match".to_string());
                }
                None
            }

            Frame::Class { class } => {
                let start = self.window_start + self.pos;
                self.ensure_buffer(1);
                let matched = if let Some((ch, len)) = self.peek_char() {
                    let in_set = class.chars.contains(&ch)
                        || class.ranges.iter().any(|(a, b)| ch >= *a && ch <= *b);
                    let ok = if class.negated { !in_set } else { in_set };
                    if ok {
                        self.consume(len);
                        self.events.push_back(ParseEvent::Token {
                            kind: TokenKind::Class(ch),
                            span: self.line_tracker.span_with_position(start, self.window_start + self.pos),
                        });
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if !matched {
                    self.fail("character class did not match".to_string());
                }
                None
            }
        }
    }

    fn fail(&mut self, msg: impl Into<String>) {
        if self.finished {
            return;
        }
        self.finished = true;
        // Find the most recent active rule from the frame stack
        let ctx = self.frames.iter().rev().find_map(|f| match f {
            Frame::Ref { name, stage, .. } if *stage != RefStage::End => Some(name.to_string()),
            _ => None,
        }).unwrap_or_else(|| self.grammar.rules.first().map(|r| r.name.clone()).unwrap_or_default());
        let msg = msg.into();
        self.events.push_back(ParseEvent::Error(
            ParseError::new(format!("failed to match: {}", msg), self.window_start + self.pos)
                .with_rule_context(ctx),
        ));
    }

    fn peek_char(&self) -> Option<(char, usize)> {
        self.input[self.pos..].chars().next().map(|ch| (ch, ch.len_utf8()))
    }

    fn consume(&mut self, bytes: usize) {
        self.pos += bytes;
    }

    /// Ensure there are at least `needed_bytes` available from current position; fill buffer if not.
    fn ensure_buffer(&mut self, needed_bytes: usize) {
        // Slide window if buffer is large and we've consumed a significant portion
        const MAX_BUFFER_SIZE: usize = 64 * 1024; // 64KB
        const MIN_SLIDE_SIZE: usize = 32 * 1024;  // Slide when we can reclaim 32KB
        
        if self.input.len() > MAX_BUFFER_SIZE && self.pos > MIN_SLIDE_SIZE {
            // Find minimum position we need to keep for backtracking
            let min_pos = self.frames.iter()
                .filter_map(|f| match f {
                    Frame::Alt { saved_pos, .. } => Some(*saved_pos),
                    Frame::Repeat { saved_pos, .. } => Some(*saved_pos),
                    _ => None,
                })
                .min()
                .unwrap_or(self.pos);
            
            // Slide window if we can reclaim space
            if min_pos > 0 {
                self.input.drain(..min_pos);
                self.window_start += min_pos;
                self.pos -= min_pos;
                
                // Adjust frame positions
                for frame in &mut self.frames {
                    match frame {
                        Frame::Alt { saved_pos, .. } => *saved_pos -= min_pos,
                        Frame::Repeat { saved_pos, .. } => *saved_pos -= min_pos,
                        _ => {}
                    }
                }
            }
        }
        
        if self.input.len().saturating_sub(self.pos) >= needed_bytes || self.eof { return; }
        if let Some(reader) = self.reader.as_mut() {
            while self.input.len().saturating_sub(self.pos) < needed_bytes {
                match reader.fill_buf() {
                    Ok(buf) => {
                        if buf.is_empty() { self.eof = true; break; }
                        let chunk = String::from_utf8_lossy(buf).to_string();
                        self.line_tracker.extend(&chunk);
                        self.input.push_str(&chunk);
                        let consumed = buf.len();
                        reader.consume(consumed);
                    }
                    Err(_) => { self.eof = true; break; }
                }
            }
        }
    }

    /// Get line and column number for a given absolute byte position.
    pub fn line_column(&self, position: usize) -> (u32, u32) {
        self.line_tracker.line_column(position)
    }

    /// Get the current absolute byte position in the input.
    pub fn current_position(&self) -> usize {
        self.window_start + self.pos
    }

    /// Create a span with position information from absolute start to end byte positions.
    pub fn span_from_range(&self, start: usize, end: usize) -> Span {
        self.line_tracker.span_with_position(start, end)
    }
}

impl<'a> Frame<'a> {
    fn from_prod(prod: &'a Prod) -> Frame<'a> {
        match prod {
            Prod::Seq(items) => Frame::Seq { items, idx: 0 },
            Prod::Alt(alts) => Frame::Alt { alts, idx: 0, saved_pos: 0 },
            Prod::Group(inner) => Frame::Group { inner },
            Prod::Repeat { item, quant } => Frame::Repeat { item, quant, count: 0, saved_pos: 0, trying: true },
            Prod::Terminal { kind, .. } => Frame::Terminal { kind },
            Prod::Class(class) => Frame::Class { class },
            Prod::Ref { name, .. } => Frame::Ref { name: name.as_str(), prod, stage: RefStage::Start },
        }
    }
}

/// Parse an input string using the given grammar, producing an iterator of events.
pub fn parse_str<'a>(grammar: &'a Grammar, input: &'a str) -> impl Iterator<Item = ParseEvent<'a>> + 'a {
    let reader = Cursor::new(input.as_bytes());
    let parser = Parser::new(grammar, reader);
    EventIter { parser }
}

/// Parse from any `BufRead`, returning an iterator of events.
pub fn parse<'a, R: BufRead + 'a>(grammar: &'a Grammar, reader: R) -> impl Iterator<Item = ParseEvent<'a>> + 'a {
    EventIter { parser: Parser::new(grammar, reader) }
}

/// Iterator adapter over `Parser::next_event`.
struct EventIter<'a, R: BufRead> {
    parser: Parser<'a, R>,
}

impl<'a, R: BufRead> Iterator for EventIter<'a, R> {
    type Item = ParseEvent<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next_event()
    }
}


