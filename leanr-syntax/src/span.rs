//! Source location tracking for error reporting

use std::fmt;
use std::sync::Arc;

/// A span in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Start byte offset
    pub start: u32,

    /// End byte offset (exclusive)
    pub end: u32,

    /// Source file ID
    pub file_id: u32,
}

impl Span {
    /// Create a new span
    pub fn new(start: u32, end: u32, file_id: u32) -> Self {
        Self { start, end, file_id }
    }

    /// Create a dummy span (for testing/generated code)
    pub fn dummy() -> Self {
        Self {
            start: 0,
            end: 0,
            file_id: u32::MAX,
        }
    }

    /// Combine two spans into a single span covering both
    pub fn to(self, other: Span) -> Span {
        debug_assert_eq!(self.file_id, other.file_id);
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            file_id: self.file_id,
        }
    }

    /// Get the length of this span
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Check if this span is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::dummy()
    }
}

/// A source file with content
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// File ID
    pub id: u32,

    /// File name/path
    pub name: String,

    /// Source content
    pub content: Arc<str>,

    /// Line start positions for error reporting
    line_starts: Vec<u32>,
}

impl SourceFile {
    /// Create a new source file
    pub fn new(id: u32, name: String, content: String) -> Self {
        let line_starts = Self::compute_line_starts(&content);
        Self {
            id,
            name,
            content: Arc::from(content),
            line_starts,
        }
    }

    /// Compute line start positions
    fn compute_line_starts(content: &str) -> Vec<u32> {
        let mut starts = vec![0];
        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                starts.push(i as u32 + 1);
            }
        }
        starts
    }

    /// Get line and column for a byte offset
    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let line = self.line_starts
            .binary_search(&offset)
            .unwrap_or_else(|i| i.saturating_sub(1));

        let line_start = self.line_starts[line];
        let col = offset - line_start;

        (line as u32, col)
    }

    /// Get the text for a span
    pub fn span_text(&self, span: Span) -> &str {
        &self.content[span.start as usize..span.end as usize]
    }

    /// Get a line of source code
    pub fn line(&self, line_num: u32) -> Option<&str> {
        let line_idx = line_num as usize;
        if line_idx >= self.line_starts.len() {
            return None;
        }

        let start = self.line_starts[line_idx] as usize;
        let end = if line_idx + 1 < self.line_starts.len() {
            self.line_starts[line_idx + 1] as usize - 1 // Exclude newline
        } else {
            self.content.len()
        };

        Some(&self.content[start..end])
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
