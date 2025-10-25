//! Lexer for Lean syntax
//!
//! Tokenizes source text into a stream of tokens with support for:
//! - Incremental lexing
//! - Unicode identifiers
//! - Comments and whitespace
//! - Numeric and string literals

use crate::span::{Span, SourceFile};
use std::fmt;

/// A token in the source code
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of token
    pub kind: TokenKind,

    /// Location in source
    pub span: Span,
}

impl Token {
    /// Create a new token
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Token types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Keywords
    Def,
    Theorem,
    Axiom,
    Inductive,
    Structure,
    Class,
    Instance,
    Let,
    In,
    Match,
    With,
    If,
    Then,
    Else,
    Fun,
    Where,
    Have,
    Show,
    By,
    Do,
    Return,

    // Special keywords
    Type,
    Prop,
    Sort,

    // Symbols
    LParen,       // (
    RParen,       // )
    LBrace,       // {
    RBrace,       // }
    LBracket,     // [
    RBracket,     // ]

    // Operators
    Colon,        // :
    ColonEq,      // :=
    Arrow,        // →  or ->
    FatArrow,     // =>
    Lambda,       // λ  or \
    Forall,       // ∀  or forall
    Exists,       // ∃  or exists
    Dot,          // .
    Comma,        // ,
    Pipe,         // |
    Underscore,   // _
    At,           // @

    // Identifiers and literals
    Ident(String),
    Number(String),
    String(String),

    // Whitespace and comments (usually skipped)
    Whitespace,
    LineComment(String),
    BlockComment(String),

    // Special
    Eof,
    Error(String),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Def => write!(f, "def"),
            TokenKind::Theorem => write!(f, "theorem"),
            TokenKind::Axiom => write!(f, "axiom"),
            TokenKind::Inductive => write!(f, "inductive"),
            TokenKind::Structure => write!(f, "structure"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::Instance => write!(f, "instance"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::With => write!(f, "with"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Fun => write!(f, "fun"),
            TokenKind::Where => write!(f, "where"),
            TokenKind::Have => write!(f, "have"),
            TokenKind::Show => write!(f, "show"),
            TokenKind::By => write!(f, "by"),
            TokenKind::Do => write!(f, "do"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Type => write!(f, "Type"),
            TokenKind::Prop => write!(f, "Prop"),
            TokenKind::Sort => write!(f, "Sort"),
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Incremental lexer
pub struct Lexer {
    /// Source file being lexed
    source: SourceFile,

    /// Current byte position
    pos: usize,

    /// Tokens produced so far (for incremental edits)
    tokens: Vec<Token>,
}

impl Lexer {
    /// Create a new lexer for a source file
    pub fn new(source: SourceFile) -> Self {
        Self {
            source,
            pos: 0,
            tokens: Vec::new(),
        }
    }

    /// Tokenize the entire source
    pub fn tokenize(mut self) -> Vec<Token> {
        while !self.is_eof() {
            let token = self.next_token();

            // Skip whitespace and comments (can be configurable)
            match token.kind {
                TokenKind::Whitespace | TokenKind::LineComment(_) | TokenKind::BlockComment(_) => {
                    continue;
                }
                _ => {
                    self.tokens.push(token);
                }
            }
        }

        // Add EOF token
        self.tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(self.pos as u32, self.pos as u32, self.source.id),
        ));

        self.tokens
    }

    /// Get the next token
    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.is_eof() {
            return Token::new(
                TokenKind::Eof,
                Span::new(self.pos as u32, self.pos as u32, self.source.id),
            );
        }

        let start = self.pos;
        let ch = self.current_char();

        let kind = match ch {
            // Single character tokens
            '(' => { self.advance(); TokenKind::LParen }
            ')' => { self.advance(); TokenKind::RParen }
            '{' => { self.advance(); TokenKind::LBrace }
            '}' => { self.advance(); TokenKind::RBrace }
            '[' => { self.advance(); TokenKind::LBracket }
            ']' => { self.advance(); TokenKind::RBracket }
            '.' => { self.advance(); TokenKind::Dot }
            ',' => { self.advance(); TokenKind::Comma }
            '|' => { self.advance(); TokenKind::Pipe }
            '_' => { self.advance(); TokenKind::Underscore }
            '@' => { self.advance(); TokenKind::At }

            // Multi-character operators
            ':' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenKind::ColonEq
                } else {
                    TokenKind::Colon
                }
            }

            '-' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    TokenKind::Arrow
                } else if self.current_char() == '-' {
                    // Line comment
                    self.advance();
                    self.lex_line_comment()
                } else {
                    TokenKind::Error("Unexpected '-'".to_string())
                }
            }

            '/' => {
                self.advance();
                if self.current_char() == '-' {
                    // Block comment
                    self.advance();
                    self.lex_block_comment()
                } else {
                    TokenKind::Error("Unexpected '/'".to_string())
                }
            }

            '=' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Error("Unexpected '='".to_string())
                }
            }

            '\\' => {
                self.advance();
                TokenKind::Lambda
            }

            // Unicode symbols
            '→' => { self.advance(); TokenKind::Arrow }
            'λ' => { self.advance(); TokenKind::Lambda }
            '∀' => { self.advance(); TokenKind::Forall }
            '∃' => { self.advance(); TokenKind::Exists }

            // String literals
            '"' => self.lex_string(),

            // Numbers
            '0'..='9' => self.lex_number(),

            // Identifiers and keywords
            _ if ch.is_alphabetic() || ch == '_' => self.lex_identifier(),

            _ => {
                self.advance();
                TokenKind::Error(format!("Unexpected character: '{}'", ch))
            }
        };

        Token::new(kind, Span::new(start as u32, self.pos as u32, self.source.id))
    }

    /// Skip whitespace
    fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.current_char().is_whitespace() {
            self.advance();
        }
    }

    /// Lex an identifier or keyword
    fn lex_identifier(&mut self) -> TokenKind {
        let start = self.pos;

        while !self.is_eof() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' || ch == '\'' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source.content[start..self.pos];

        // Check for keywords
        match text {
            "def" => TokenKind::Def,
            "theorem" => TokenKind::Theorem,
            "axiom" => TokenKind::Axiom,
            "inductive" => TokenKind::Inductive,
            "structure" => TokenKind::Structure,
            "class" => TokenKind::Class,
            "instance" => TokenKind::Instance,
            "let" => TokenKind::Let,
            "in" => TokenKind::In,
            "match" => TokenKind::Match,
            "with" => TokenKind::With,
            "if" => TokenKind::If,
            "then" => TokenKind::Then,
            "else" => TokenKind::Else,
            "fun" => TokenKind::Fun,
            "where" => TokenKind::Where,
            "have" => TokenKind::Have,
            "show" => TokenKind::Show,
            "by" => TokenKind::By,
            "do" => TokenKind::Do,
            "return" => TokenKind::Return,
            "Type" => TokenKind::Type,
            "Prop" => TokenKind::Prop,
            "Sort" => TokenKind::Sort,
            "forall" => TokenKind::Forall,
            "exists" => TokenKind::Exists,
            _ => TokenKind::Ident(text.to_string()),
        }
    }

    /// Lex a number
    fn lex_number(&mut self) -> TokenKind {
        let start = self.pos;

        while !self.is_eof() && self.current_char().is_ascii_digit() {
            self.advance();
        }

        let text = &self.source.content[start..self.pos];
        TokenKind::Number(text.to_string())
    }

    /// Lex a string literal
    fn lex_string(&mut self) -> TokenKind {
        self.advance(); // Skip opening "
        let start = self.pos;

        while !self.is_eof() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance(); // Skip escape char
                if !self.is_eof() {
                    self.advance(); // Skip escaped char
                }
            } else {
                self.advance();
            }
        }

        if self.is_eof() {
            return TokenKind::Error("Unterminated string".to_string());
        }

        let text = self.source.content[start..self.pos].to_string();
        self.advance(); // Skip closing "

        TokenKind::String(text)
    }

    /// Lex a line comment
    fn lex_line_comment(&mut self) -> TokenKind {
        let start = self.pos;

        while !self.is_eof() && self.current_char() != '\n' {
            self.advance();
        }

        let text = self.source.content[start..self.pos].to_string();
        TokenKind::LineComment(text)
    }

    /// Lex a block comment
    fn lex_block_comment(&mut self) -> TokenKind {
        let start = self.pos;
        let mut depth = 1;

        while !self.is_eof() && depth > 0 {
            if self.current_char() == '/' && self.peek_char() == Some('-') {
                self.advance();
                self.advance();
                depth += 1;
            } else if self.current_char() == '-' && self.peek_char() == Some('/') {
                self.advance();
                self.advance();
                depth -= 1;
            } else {
                self.advance();
            }
        }

        let text = self.source.content[start..self.pos].to_string();

        if depth > 0 {
            TokenKind::Error("Unterminated block comment".to_string())
        } else {
            TokenKind::BlockComment(text)
        }
    }

    /// Get current character
    fn current_char(&self) -> char {
        self.source.content[self.pos..].chars().next().unwrap_or('\0')
    }

    /// Peek at next character
    fn peek_char(&self) -> Option<char> {
        self.source.content[self.pos..].chars().nth(1)
    }

    /// Advance to next character
    fn advance(&mut self) {
        if let Some(ch) = self.source.content[self.pos..].chars().next() {
            self.pos += ch.len_utf8();
        }
    }

    /// Check if at end of file
    fn is_eof(&self) -> bool {
        self.pos >= self.source.content.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<TokenKind> {
        let source = SourceFile::new(0, "test.lean".to_string(), input.to_string());
        let lexer = Lexer::new(source);
        lexer.tokenize().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_keywords() {
        let tokens = lex("def theorem inductive");
        assert_eq!(tokens, vec![
            TokenKind::Def,
            TokenKind::Theorem,
            TokenKind::Inductive,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_symbols() {
        let tokens = lex("( ) { } : := ->");
        assert_eq!(tokens, vec![
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Colon,
            TokenKind::ColonEq,
            TokenKind::Arrow,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_identifiers() {
        let tokens = lex("foo bar_baz x'");
        assert_eq!(tokens, vec![
            TokenKind::Ident("foo".to_string()),
            TokenKind::Ident("bar_baz".to_string()),
            TokenKind::Ident("x'".to_string()),
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_numbers() {
        let tokens = lex("42 0 123");
        assert_eq!(tokens, vec![
            TokenKind::Number("42".to_string()),
            TokenKind::Number("0".to_string()),
            TokenKind::Number("123".to_string()),
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_simple_def() {
        let tokens = lex("def id (x : Nat) : Nat := x");
        assert_eq!(tokens[0], TokenKind::Def);
        assert_eq!(tokens[1], TokenKind::Ident("id".to_string()));
        assert_eq!(tokens[2], TokenKind::LParen);
    }
}
