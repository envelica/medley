//! Source span for position tracking.

/// A byte-span with position information (for diagnostics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    /// Optional line number (1-indexed) for diagnostics
    pub line: Option<u32>,
    /// Optional column number (1-indexed) for diagnostics
    pub column: Option<u32>,
}

impl Span {
    /// Create a new span from byte positions only
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            line: None,
            column: None,
        }
    }

    /// Create a span with line/column information
    pub fn with_position(start: usize, end: usize, line: u32, column: u32) -> Self {
        Self {
            start,
            end,
            line: Some(line),
            column: Some(column),
        }
    }
}
