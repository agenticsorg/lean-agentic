//! Parser for Lean surface syntax
//!
//! Recursive descent parser with error recovery.

use crate::ast::*;
use crate::lexer::{Token, TokenKind};
use crate::span::Span;
use std::fmt;

/// Parse error
#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl ParseError {
    pub fn new(span: Span, message: String) -> Self {
        Self { span, message }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at {}: {}", self.span, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Recursive descent parser
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create a new parser from tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse a list of declarations
    pub fn parse_decls(&mut self) -> crate::Result<Vec<Decl>> {
        let mut decls = Vec::new();

        while !self.is_eof() {
            decls.push(self.parse_decl()?);
        }

        Ok(decls)
    }

    /// Parse a single declaration
    pub fn parse_decl(&mut self) -> crate::Result<Decl> {
        let token = self.current();

        match &token.kind {
            TokenKind::Def => Ok(Decl::Def(self.parse_def()?)),
            TokenKind::Theorem => Ok(Decl::Theorem(self.parse_theorem()?)),
            TokenKind::Axiom => Ok(Decl::Axiom(self.parse_axiom()?)),
            TokenKind::Inductive => Ok(Decl::Inductive(self.parse_inductive()?)),
            TokenKind::Structure => Ok(Decl::Structure(self.parse_structure()?)),
            _ => Err(ParseError::new(
                token.span,
                format!("Expected declaration, found {:?}", token.kind),
            )),
        }
    }

    /// Parse def declaration: def name params : type := body
    fn parse_def(&mut self) -> crate::Result<DefDecl> {
        let start = self.expect(TokenKind::Def)?.span;

        let name = self.parse_ident()?;
        let universe_params = self.parse_universe_params()?;
        let params = self.parse_params()?;

        let return_type = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        self.expect(TokenKind::ColonEq)?;
        let body = Box::new(self.parse_expr()?);

        let end = body.span();

        Ok(DefDecl {
            span: start.to(end),
            name,
            universe_params,
            params,
            return_type,
            body,
        })
    }

    /// Parse theorem declaration
    fn parse_theorem(&mut self) -> crate::Result<TheoremDecl> {
        let start = self.expect(TokenKind::Theorem)?.span;

        let name = self.parse_ident()?;
        let universe_params = self.parse_universe_params()?;
        let params = self.parse_params()?;

        self.expect(TokenKind::Colon)?;
        let type_ = Box::new(self.parse_expr()?);

        self.expect(TokenKind::ColonEq)?;
        let proof = Box::new(self.parse_expr()?);

        let end = proof.span();

        Ok(TheoremDecl {
            span: start.to(end),
            name,
            universe_params,
            params,
            type_,
            proof,
        })
    }

    /// Parse axiom declaration
    fn parse_axiom(&mut self) -> crate::Result<AxiomDecl> {
        let start = self.expect(TokenKind::Axiom)?.span;

        let name = self.parse_ident()?;
        let universe_params = self.parse_universe_params()?;
        let params = self.parse_params()?;

        self.expect(TokenKind::Colon)?;
        let type_ = Box::new(self.parse_expr()?);

        let end = type_.span();

        Ok(AxiomDecl {
            span: start.to(end),
            name,
            universe_params,
            params,
            type_,
        })
    }

    /// Parse inductive declaration
    fn parse_inductive(&mut self) -> crate::Result<InductiveDecl> {
        let start = self.expect(TokenKind::Inductive)?.span;

        let name = self.parse_ident()?;
        let universe_params = self.parse_universe_params()?;
        let params = self.parse_params()?;

        let type_ = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        self.expect(TokenKind::Where)?;

        let mut constructors = Vec::new();
        while self.check(&TokenKind::Pipe) {
            self.advance();
            constructors.push(self.parse_constructor()?);
        }

        let end = if let Some(last) = constructors.last() {
            last.span
        } else {
            name.span
        };

        Ok(InductiveDecl {
            span: start.to(end),
            name,
            universe_params,
            params,
            type_,
            constructors,
        })
    }

    /// Parse a constructor
    fn parse_constructor(&mut self) -> crate::Result<Constructor> {
        let name = self.parse_ident()?;
        let params = self.parse_params()?;

        let type_ = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        let end = type_.as_ref().map(|t| t.span()).unwrap_or(name.span);

        Ok(Constructor {
            span: name.span.to(end),
            name,
            params,
            type_,
        })
    }

    /// Parse structure declaration
    fn parse_structure(&mut self) -> crate::Result<StructureDecl> {
        let start = self.expect(TokenKind::Structure)?.span;

        let name = self.parse_ident()?;
        let universe_params = self.parse_universe_params()?;
        let params = self.parse_params()?;

        let extends = Vec::new(); // TODO: Parse extends

        self.expect(TokenKind::Where)?;

        let mut fields = Vec::new();
        while !self.is_eof() && !self.check(&TokenKind::RBrace) {
            let field_name = self.parse_ident()?;
            self.expect(TokenKind::Colon)?;
            let field_type = Box::new(self.parse_expr()?);

            fields.push(Field {
                span: field_name.span.to(field_type.span()),
                name: field_name,
                type_: field_type,
            });

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        let end = if let Some(last) = fields.last() {
            last.span
        } else {
            name.span
        };

        Ok(StructureDecl {
            span: start.to(end),
            name,
            universe_params,
            params,
            extends,
            fields,
        })
    }

    /// Parse universe parameters: .{u v}
    fn parse_universe_params(&mut self) -> crate::Result<Vec<Ident>> {
        let mut params = Vec::new();

        if self.check(&TokenKind::Dot) {
            self.advance();
            self.expect(TokenKind::LBrace)?;

            while !self.check(&TokenKind::RBrace) {
                params.push(self.parse_ident()?);
            }

            self.expect(TokenKind::RBrace)?;
        }

        Ok(params)
    }

    /// Parse function parameters
    fn parse_params(&mut self) -> crate::Result<Vec<Param>> {
        let mut params = Vec::new();

        while self.check(&TokenKind::LParen) || self.check(&TokenKind::LBrace) {
            let implicit = self.check(&TokenKind::LBrace);
            let start = self.current().span;
            self.advance();

            let mut names = vec![self.parse_ident()?];

            // Multiple names: (x y z : T)
            while !self.check(&TokenKind::Colon) && !self.is_eof() {
                names.push(self.parse_ident()?);
            }

            let type_ = if self.check(&TokenKind::Colon) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };

            let end_token = if implicit {
                TokenKind::RBrace
            } else {
                TokenKind::RParen
            };

            let end = self.expect(end_token)?.span;

            params.push(Param {
                span: start.to(end),
                names,
                type_,
                implicit,
            });
        }

        Ok(params)
    }

    /// Parse an expression
    pub fn parse_expr(&mut self) -> crate::Result<Expr> {
        self.parse_arrow_expr()
    }

    /// Parse arrow type: A -> B
    fn parse_arrow_expr(&mut self) -> crate::Result<Expr> {
        let mut expr = self.parse_forall_expr()?;

        while self.check(&TokenKind::Arrow) {
            let arrow_span = self.advance().span;
            let to = self.parse_forall_expr()?;
            let span = expr.span().to(to.span());

            expr = Expr::Arrow {
                span,
                from: Box::new(expr),
                to: Box::new(to),
            };
        }

        Ok(expr)
    }

    /// Parse forall: forall x : T, U
    fn parse_forall_expr(&mut self) -> crate::Result<Expr> {
        if self.check(&TokenKind::Forall) {
            let start = self.advance().span;
            let params = self.parse_params()?;
            self.expect(TokenKind::Comma)?;
            let body = Box::new(self.parse_expr()?);
            let span = start.to(body.span());

            Ok(Expr::Forall { span, params, body })
        } else {
            self.parse_lambda_expr()
        }
    }

    /// Parse lambda: fun x => body
    fn parse_lambda_expr(&mut self) -> crate::Result<Expr> {
        if self.check(&TokenKind::Fun) || self.check(&TokenKind::Lambda) {
            let start = self.advance().span;
            let params = self.parse_lambda_params()?;
            self.expect(TokenKind::FatArrow)?;
            let body = Box::new(self.parse_expr()?);
            let span = start.to(body.span());

            Ok(Expr::Lam { span, params, body })
        } else {
            self.parse_let_expr()
        }
    }

    /// Parse lambda parameters (simpler than def params)
    fn parse_lambda_params(&mut self) -> crate::Result<Vec<Param>> {
        let mut params = Vec::new();

        loop {
            if self.check(&TokenKind::FatArrow) {
                break;
            }

            let implicit = self.check(&TokenKind::LBrace);

            if implicit {
                self.advance();
                let name = self.parse_ident()?;
                let type_ = if self.check(&TokenKind::Colon) {
                    self.advance();
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                self.expect(TokenKind::RBrace)?;

                params.push(Param {
                    span: name.span,
                    names: vec![name],
                    type_,
                    implicit: true,
                });
            } else if self.check(&TokenKind::LParen) {
                self.advance();
                let name = self.parse_ident()?;
                let type_ = if self.check(&TokenKind::Colon) {
                    self.advance();
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                self.expect(TokenKind::RParen)?;

                params.push(Param {
                    span: name.span,
                    names: vec![name],
                    type_,
                    implicit: false,
                });
            } else {
                // Simple name without parens
                let name = self.parse_ident()?;
                params.push(Param {
                    span: name.span,
                    names: vec![name],
                    type_: None,
                    implicit: false,
                });
            }
        }

        Ok(params)
    }

    /// Parse let expression
    fn parse_let_expr(&mut self) -> crate::Result<Expr> {
        if self.check(&TokenKind::Let) {
            let start = self.advance().span;
            let name = self.parse_ident()?;

            let type_ = if self.check(&TokenKind::Colon) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };

            self.expect(TokenKind::ColonEq)?;
            let value = Box::new(self.parse_expr()?);

            self.expect(TokenKind::In)?;
            let body = Box::new(self.parse_expr()?);

            let span = start.to(body.span());

            Ok(Expr::Let {
                span,
                name,
                type_,
                value,
                body,
            })
        } else {
            self.parse_match_expr()
        }
    }

    /// Parse match expression
    fn parse_match_expr(&mut self) -> crate::Result<Expr> {
        if self.check(&TokenKind::Match) {
            let start = self.advance().span;
            let scrutinee = Box::new(self.parse_app_expr()?);

            self.expect(TokenKind::With)?;

            let mut arms = Vec::new();
            while self.check(&TokenKind::Pipe) {
                self.advance();
                let pattern = self.parse_pattern()?;
                self.expect(TokenKind::FatArrow)?;
                let body = Box::new(self.parse_expr()?);

                arms.push(MatchArm {
                    span: pattern.span().to(body.span()),
                    pattern,
                    body,
                });
            }

            let end = arms.last().map(|a| a.span).unwrap_or(scrutinee.span());

            Ok(Expr::Match {
                span: start.to(end),
                scrutinee,
                arms,
            })
        } else {
            self.parse_app_expr()
        }
    }

    /// Parse application: f x y z
    fn parse_app_expr(&mut self) -> crate::Result<Expr> {
        let mut func = self.parse_atomic_expr()?;
        let mut args = Vec::new();

        while !self.is_eof() && self.is_atomic_start() {
            args.push(self.parse_atomic_expr()?);
        }

        if args.is_empty() {
            Ok(func)
        } else {
            let span = func.span().to(args.last().unwrap().span());
            Ok(Expr::App {
                span,
                func: Box::new(func),
                args,
            })
        }
    }

    /// Parse atomic (primary) expression
    fn parse_atomic_expr(&mut self) -> crate::Result<Expr> {
        let token = self.current();

        match &token.kind {
            TokenKind::Ident(name) => {
                let span = token.span;
                self.advance();
                Ok(Expr::Ident(Ident::new(name.clone(), span)))
            }

            TokenKind::Number(n) => {
                let span = token.span;
                self.advance();
                let num = n.parse::<u64>().map_err(|_| {
                    ParseError::new(span, "Invalid number".to_string())
                })?;
                Ok(Expr::Lit(LitExpr {
                    span,
                    kind: LitKind::Nat(num),
                }))
            }

            TokenKind::String(s) => {
                let span = token.span;
                self.advance();
                Ok(Expr::Lit(LitExpr {
                    span,
                    kind: LitKind::String(s.clone()),
                }))
            }

            TokenKind::Underscore => {
                let span = token.span;
                self.advance();
                Ok(Expr::Hole { span })
            }

            TokenKind::Type => {
                let span = token.span;
                self.advance();
                Ok(Expr::Universe {
                    span,
                    kind: UniverseKind::Type,
                })
            }

            TokenKind::Prop => {
                let span = token.span;
                self.advance();
                Ok(Expr::Universe {
                    span,
                    kind: UniverseKind::Prop,
                })
            }

            TokenKind::LParen => {
                let start = self.advance().span;
                let expr = self.parse_expr()?;
                let end = self.expect(TokenKind::RParen)?.span;
                Ok(Expr::Paren {
                    span: start.to(end),
                    expr: Box::new(expr),
                })
            }

            _ => Err(ParseError::new(
                token.span,
                format!("Expected expression, found {:?}", token.kind),
            )),
        }
    }

    /// Parse a pattern
    fn parse_pattern(&mut self) -> crate::Result<Pattern> {
        let token = self.current();

        match &token.kind {
            TokenKind::Underscore => {
                let span = token.span;
                self.advance();
                Ok(Pattern::Wildcard { span })
            }

            TokenKind::Number(n) => {
                let span = token.span;
                self.advance();
                let num = n.parse::<u64>().map_err(|_| {
                    ParseError::new(span, "Invalid number in pattern".to_string())
                })?;
                Ok(Pattern::Lit {
                    span,
                    lit: LitKind::Nat(num),
                })
            }

            TokenKind::Ident(name) => {
                let span = token.span;
                let ident = Ident::new(name.clone(), span);
                self.advance();

                // Check if this is a constructor with args
                let mut args = Vec::new();
                while !self.is_eof() && self.is_pattern_start() && !self.check(&TokenKind::FatArrow) {
                    args.push(self.parse_pattern()?);
                }

                if args.is_empty() {
                    // Simple variable
                    Ok(Pattern::Var { span, name: ident })
                } else {
                    // Constructor pattern
                    let end = args.last().unwrap().span();
                    Ok(Pattern::Constructor {
                        span: span.to(end),
                        name: ident,
                        args,
                    })
                }
            }

            _ => Err(ParseError::new(
                token.span,
                format!("Expected pattern, found {:?}", token.kind),
            )),
        }
    }

    /// Parse an identifier
    fn parse_ident(&mut self) -> crate::Result<Ident> {
        match &self.current().kind {
            TokenKind::Ident(name) => {
                let span = self.current().span;
                let name = name.clone();
                self.advance();
                Ok(Ident::new(name, span))
            }
            _ => Err(ParseError::new(
                self.current().span,
                format!("Expected identifier, found {:?}", self.current().kind),
            )),
        }
    }

    /// Check if current token matches
    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_eof() {
            return false;
        }
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    /// Expect a specific token
    fn expect(&mut self, kind: TokenKind) -> crate::Result<Token> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(
                self.current().span,
                format!("Expected {:?}, found {:?}", kind, self.current().kind),
            ))
        }
    }

    /// Get current token
    fn current(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    /// Advance to next token
    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    /// Check if at end of input
    fn is_eof(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    /// Check if current token can start an atomic expression
    fn is_atomic_start(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Ident(_)
                | TokenKind::Number(_)
                | TokenKind::String(_)
                | TokenKind::Underscore
                | TokenKind::Type
                | TokenKind::Prop
                | TokenKind::LParen
        )
    }

    /// Check if current token can start a pattern
    fn is_pattern_start(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Ident(_) | TokenKind::Number(_) | TokenKind::Underscore
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::span::SourceFile;

    fn parse(input: &str) -> crate::Result<Vec<Decl>> {
        let source = SourceFile::new(0, "test.lean".to_string(), input.to_string());
        let tokens = Lexer::new(source).tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse_decls()
    }

    #[test]
    fn test_simple_def() {
        let input = "def id (x : Nat) : Nat := x";
        let decls = parse(input).unwrap();
        assert_eq!(decls.len(), 1);

        match &decls[0] {
            Decl::Def(def) => {
                assert_eq!(def.name.name, "id");
                assert_eq!(def.params.len(), 1);
            }
            _ => panic!("Expected def"),
        }
    }

    #[test]
    fn test_lambda() {
        let input = "def test := fun x => x";
        let decls = parse(input).unwrap();
        assert_eq!(decls.len(), 1);
    }

    #[test]
    fn test_inductive() {
        let input = r#"
            inductive Nat where
            | zero : Nat
            | succ (n : Nat) : Nat
        "#;
        let decls = parse(input).unwrap();
        assert_eq!(decls.len(), 1);

        match &decls[0] {
            Decl::Inductive(ind) => {
                assert_eq!(ind.name.name, "Nat");
                assert_eq!(ind.constructors.len(), 2);
            }
            _ => panic!("Expected inductive"),
        }
    }
}
