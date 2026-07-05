use tails::compiler::lexer::tokenize;
use tails::compiler::parser::{parse, AstNode, BindingPattern, Expression, Statement};

#[test]
fn test_number_literal() {
    let mut tokens = tokenize("42").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::Expression(expr) => {
                    assert!(matches!(expr, Expression::NumberLiteral(42.0)));
                }
                _ => panic!("Expected expression statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_string_literal() {
    let mut tokens = tokenize(r#""hello""#).unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::Expression(expr) => {
                    assert!(matches!(expr, Expression::StringLiteral(s) if s == "hello"));
                }
                _ => panic!("Expected expression statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_binary_operation() {
    let mut tokens = tokenize("2 + 3").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::Expression(expr) => {
                    assert!(matches!(expr, Expression::BinaryOp { .. }));
                }
                _ => panic!("Expected expression statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_variable_declaration() {
    let mut tokens = tokenize("const x = 42;").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::VariableDeclaration { declarations, .. } => {
                    assert_eq!(declarations.len(), 1);
                    match &declarations[0].id {
                        BindingPattern::Identifier(name) => assert_eq!(name, "x"),
                        _ => panic!("Expected identifier pattern"),
                    }
                    assert!(declarations[0].init.is_some());
                }
                _ => panic!("Expected variable declaration"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_function_declaration() {
    let mut tokens = tokenize("function add(a, b) { return a + b; }").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::FunctionDeclaration {
                    name, params, body, ..
                } => {
                    assert_eq!(name, "add");
                    assert_eq!(params, &vec!["a".to_string(), "b".to_string()]);
                    assert_eq!(body.len(), 1);
                }
                _ => panic!("Expected function declaration"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_if_statement() {
    let mut tokens = tokenize("if (true) { 1 } else { 2 }").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::IfStatement {
                    condition,
                    consequent: _,
                    alternate,
                } => {
                    assert!(matches!(condition, Expression::BooleanLiteral(true)));
                    assert!(alternate.is_some());
                }
                _ => panic!("Expected if statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_while_statement() {
    let mut tokens = tokenize("while (true) { 1 }").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::WhileStatement { condition, .. } => {
                    assert!(matches!(condition, Expression::BooleanLiteral(true)));
                }
                _ => panic!("Expected while statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_assignment() {
    let mut tokens = tokenize("x = 42").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::Expression(expr) => {
                    assert!(matches!(expr, Expression::Assignment { .. }));
                }
                _ => panic!("Expected expression statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_function_call() {
    let mut tokens = tokenize("add(1, 2)").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::Expression(expr) => {
                    assert!(matches!(expr, Expression::Call { .. }));
                }
                _ => panic!("Expected expression statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_member_access() {
    let mut tokens = tokenize("obj.prop").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::Expression(expr) => {
                    assert!(matches!(
                        expr,
                        Expression::Member {
                            computed: false,
                            ..
                        }
                    ));
                }
                _ => panic!("Expected expression statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_computed_member_access() {
    let mut tokens = tokenize("obj[\"prop\"]").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::Expression(expr) => {
                    assert!(matches!(expr, Expression::Member { computed: true, .. }));
                }
                _ => panic!("Expected expression statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_unary_operation() {
    let mut tokens = tokenize("-5").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::Expression(expr) => {
                    assert!(matches!(expr, Expression::UnaryOp { .. }));
                }
                _ => panic!("Expected expression statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_complex_expression() {
    let mut tokens = tokenize("(2 + 3) * 4").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::Expression(expr) => {
                    assert!(matches!(expr, Expression::BinaryOp { .. }));
                }
                _ => panic!("Expected expression statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_multiple_statements() {
    let mut tokens = tokenize("1; 2; 3").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 3);
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_block_statement() {
    let mut tokens = tokenize("{ 1; 2; 3 }").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::BlockStatement(inner) => {
                    assert_eq!(inner.len(), 3);
                }
                _ => panic!("Expected block statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_return_statement() {
    let mut tokens = tokenize("return 42;").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::ReturnStatement(value) => {
                    assert!(value.is_some());
                }
                _ => panic!("Expected return statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_return_without_value() {
    let mut tokens = tokenize("return;").unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0].inner {
                Statement::ReturnStatement(value) => {
                    assert!(value.is_none());
                }
                _ => panic!("Expected return statement"),
            }
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_template_literal_with_postfix_increment() {
    // Regression: a template expression whose last token is `++` used to
    // spin the parser forever because the lexer stripped the trailing Eof
    // and `advance` clamped pos at the last index, so `parse_postfix` kept
    // matching Token::Increment and never advanced.
    let source = r#"
        let counter = 0;
        for (const key of obj) {
            ids[key] = `key_${counter++}`;
        }
    "#;
    let mut tokens = tokenize(source).unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            // `let counter = 0;` + the for-of statement
            assert_eq!(stmts.len(), 2);
            assert!(matches!(stmts[1].inner, Statement::ForOfStatement { .. }));
        }
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_complex_program() {
    let source = r#"
        const x = 42;
        const y = "hello";
        
        function add(a, b) {
            return a + b;
        }
        
        if (x > 10) {
            add(x, 1)
        } else {
            0
        }
    "#;

    let mut tokens = tokenize(source).unwrap();
    let ast = parse(&mut tokens).unwrap();

    match ast {
        AstNode::Program(stmts) => {
            assert!(stmts.len() >= 3);
        }
        _ => panic!("Expected program"),
    }
}
