//! Test elaboration of simple definitions

#[cfg(test)]
mod tests {
    use leanr_syntax::{Lexer, Parser};
    use leanr_syntax::span::SourceFile;
    use leanr_elab::elaborate_decl;
    use lean_agentic::{Arena, Environment};

    fn setup() -> (Arena, Environment) {
        (Arena::new(), Environment::new())
    }

    fn parse_and_elaborate(source: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (mut arena, mut env) = setup();

        let source_file = SourceFile::new(0, "test.lean".to_string(), source.to_string());
        let tokens = Lexer::new(source_file).tokenize();
        let mut parser = Parser::new(tokens);
        let decls = parser.parse_decls()?;

        for decl in &decls {
            elaborate_decl(decl, &mut arena, &mut env)?;
        }

        Ok(())
    }

    #[test]
    fn test_identity_function() {
        let source = "def id (α : Type) (x : α) : α := x";
        parse_and_elaborate(source).unwrap();
    }

    #[test]
    fn test_constant_function() {
        let source = "def const (α β : Type) (x : α) (y : β) : α := x";
        parse_and_elaborate(source).unwrap();
    }

    #[test]
    fn test_lambda_expr() {
        let source = "def double := fun (n : Nat) => n";
        parse_and_elaborate(source).unwrap();
    }

    #[test]
    fn test_let_binding() {
        let source = r#"
            def test (x : Nat) : Nat :=
              let y := x in y
        "#;
        parse_and_elaborate(source).unwrap();
    }
}
