use tails::compiler::lexer::{tokenize, TemplatePart, Token};

#[test]
fn test_numbers() {
    let tokens = tokenize("42").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Number(42.0));
}

#[test]
#[allow(clippy::approx_constant)]
fn test_float_numbers() {
    let tokens = tokenize("3.14").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Number(3.14));
}

#[test]
fn test_strings() {
    let tokens = tokenize(r#""hello""#).unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::String("hello".to_string()));
}

#[test]
fn test_identifiers() {
    let tokens = tokenize("myVar").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Identifier("myVar".to_string()));
}

#[test]
fn test_keywords() {
    let tokens = tokenize("const let var function return").unwrap();
    assert_eq!(tokens.len(), 6);
    assert_eq!(tokens[0].token, Token::Const);
    assert_eq!(tokens[1].token, Token::Let);
    assert_eq!(tokens[2].token, Token::Var);
    assert_eq!(tokens[3].token, Token::Function);
    assert_eq!(tokens[4].token, Token::Return);
}

#[test]
fn test_operators() {
    let tokens = tokenize("+ - * 0 / % 0 = == === != !== < > <= >=").unwrap();
    // + - * 0 / % 0 = == === != !== < > <= >= Eof = 17 tokens
    assert_eq!(tokens.len(), 17);
    assert_eq!(tokens[0].token, Token::Plus);
    assert_eq!(tokens[1].token, Token::Minus);
    assert_eq!(tokens[2].token, Token::Star);
    assert_eq!(tokens[3].token, Token::Number(0.0));
    assert_eq!(tokens[4].token, Token::Slash);
    assert_eq!(tokens[5].token, Token::Percent);
    assert_eq!(tokens[6].token, Token::Number(0.0));
    assert_eq!(tokens[7].token, Token::Assign);
    assert_eq!(tokens[8].token, Token::Equal);
    assert_eq!(tokens[9].token, Token::StrictEqual);
    assert_eq!(tokens[10].token, Token::NotEqual);
    assert_eq!(tokens[11].token, Token::StrictNotEqual);
    assert_eq!(tokens[12].token, Token::Less);
    assert_eq!(tokens[13].token, Token::Greater);
    assert_eq!(tokens[14].token, Token::LessEqual);
    assert_eq!(tokens[15].token, Token::GreaterEqual);
    assert_eq!(tokens[16].token, Token::Eof);
}

#[test]
fn test_punctuation() {
    let tokens = tokenize("( ) { } [ ] ; : , . ?").unwrap();
    // ( ) { } [ ] ; : , . ? Eof = 12 tokens
    assert_eq!(tokens.len(), 12);
    assert_eq!(tokens[0].token, Token::LeftParen);
    assert_eq!(tokens[1].token, Token::RightParen);
    assert_eq!(tokens[2].token, Token::LeftBrace);
    assert_eq!(tokens[3].token, Token::RightBrace);
    assert_eq!(tokens[4].token, Token::LeftBracket);
    assert_eq!(tokens[5].token, Token::RightBracket);
    assert_eq!(tokens[6].token, Token::Semicolon);
    assert_eq!(tokens[7].token, Token::Colon);
    assert_eq!(tokens[8].token, Token::Comma);
    assert_eq!(tokens[9].token, Token::Dot);
    assert_eq!(tokens[10].token, Token::Question);
    assert_eq!(tokens[11].token, Token::Eof);
}

#[test]
fn test_comments() {
    let tokens = tokenize("42 // this is a comment").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Number(42.0));
}

#[test]
fn test_multiline_comment() {
    let tokens = tokenize("42 /* multi\nline\ncomment */ 100").unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].token, Token::Number(42.0));
    assert_eq!(tokens[1].token, Token::Number(100.0));
}

#[test]
fn test_complex_expression() {
    let tokens = tokenize("2 + 3 * 4").unwrap();
    // 2 + 3 * 4 Eof = 6 tokens
    assert_eq!(tokens.len(), 6);
    assert_eq!(tokens[0].token, Token::Number(2.0));
    assert_eq!(tokens[1].token, Token::Plus);
    assert_eq!(tokens[2].token, Token::Number(3.0));
    assert_eq!(tokens[3].token, Token::Star);
    assert_eq!(tokens[4].token, Token::Number(4.0));
    assert_eq!(tokens[5].token, Token::Eof);
}

#[test]
fn test_string_with_escape() {
    let tokens = tokenize(r#""hello\nworld""#).unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::String("hello\nworld".to_string()));
}

#[test]
fn test_empty_string() {
    let tokens = tokenize(r#""""#).unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::String("".to_string()));
}

#[test]
fn test_null_literal() {
    let tokens = tokenize("null").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Identifier("null".to_string()));
}

#[test]
fn test_undefined_literal() {
    let tokens = tokenize("undefined").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Identifier("undefined".to_string()));
}

#[test]
fn test_boolean_literals() {
    let tokens = tokenize("true false").unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].token, Token::Identifier("true".to_string()));
    assert_eq!(tokens[1].token, Token::Identifier("false".to_string()));
}

#[test]
fn test_arrow_function() {
    let tokens = tokenize("=>").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Arrow);
}

#[test]
fn test_spread_operator() {
    let tokens = tokenize("...").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Ellipsis);
}

#[test]
fn test_template_string() {
    let tokens = tokenize(r#"`hello ${name}`"#).unwrap();
    assert_eq!(tokens.len(), 2);
    match &tokens[0].token {
        Token::TemplateLiteral(parts) => {
            assert_eq!(parts.len(), 2);
            match &parts[0] {
                TemplatePart::Text(text) => assert_eq!(text, "hello "),
                _ => panic!("Expected text part"),
            }
            match &parts[1] {
                TemplatePart::Expression(tokens) => {
                    // Identifier followed by the trailing Eof kept by the lexer
                    // (removing it caused infinite loops in the parser; see
                    // `tokenize_template_literal`).
                    assert_eq!(tokens.len(), 2);
                    assert_eq!(tokens[0].token, Token::Identifier("name".to_string()));
                    assert_eq!(tokens[1].token, Token::Eof);
                }
                _ => panic!("Expected expression part"),
            }
        }
        _ => panic!("Expected TemplateLiteral"),
    }
    assert_eq!(tokens[1].token, Token::Eof);
}

#[test]
fn test_unterminated_string() {
    let result = tokenize(r#""hello"#);
    assert!(result.is_err());
}

#[test]
fn test_unterminated_comment() {
    let result = tokenize("/* unterminated");
    assert!(result.is_err());
}

#[test]
fn test_complex_program() {
    let source = r#"
        const x = 42;
        const y = "hello";

        function add(a, b) {
            return a + b;
        }

        add(x, y.length)
    "#;

    let tokens = tokenize(source).unwrap();
    assert!(tokens.len() > 20);
}
