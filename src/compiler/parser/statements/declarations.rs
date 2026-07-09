use super::super::*;
use crate::errors::{Error, Result};
use crate::well_known as wk;

impl<'a> Parser<'a> {
    pub(crate) fn parse_variable_declaration(&mut self) -> Result<SpannedNode<Statement>> {
        let kind = match self.advance().token {
            Token::Var => VarKind::Var,
            Token::Let => VarKind::Let,
            Token::Const => VarKind::Const,
            _ => unreachable!(),
        };
        let mut declarations = Vec::new();
        loop {
            let id = self.parse_binding_pattern()?;
            let type_annotation = if self.peek().token == Token::Colon {
                self.advance();
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
            let init = if self.peek().token == Token::Assign {
                self.advance();
                Some(self.parse_expression()?.inner)
            } else {
                None
            };
            declarations.push(VariableDeclarator {
                id,
                type_annotation,
                init,
            });
            if self.peek().token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect_statement_semicolon()?;
        Ok(self.spanned(Statement::VariableDeclaration { kind, declarations }))
    }

    pub(crate) fn parse_binding_pattern(&mut self) -> Result<BindingPattern> {
        match self.peek().token.clone() {
            Token::LeftBracket => self.parse_array_binding_pattern(),
            Token::LeftBrace => self.parse_object_binding_pattern(),
            _ => {
                let id = match self.advance().token {
                    Token::Identifier(name) => name,
                    Token::Get => "get".to_string(),
                    Token::Set => "set".to_string(),
                    Token::Delete => "delete".to_string(),
                    Token::New => "new".to_string(),
                    Token::This => "this".to_string(),
                    Token::Return => "return".to_string(),
                    Token::If => "if".to_string(),
                    Token::Else => "else".to_string(),
                    Token::While => "while".to_string(),
                    Token::For => "for".to_string(),
                    Token::Do => "do".to_string(),
                    Token::Function => "function".to_string(),
                    Token::Class => "class".to_string(),
                    Token::Switch => "switch".to_string(),
                    Token::Case => "case".to_string(),
                    Token::Break => "break".to_string(),
                    Token::Continue => "continue".to_string(),
                    Token::Typeof => "typeof".to_string(),
                    Token::Instanceof => "instanceof".to_string(),
                    Token::In => "in".to_string(),
                    Token::Void => "void".to_string(),
                    Token::Catch => wk::CATCH.to_string(),
                    Token::Finally => wk::FINALLY.to_string(),
                    Token::Throw => "throw".to_string(),
                    Token::Async => "async".to_string(),
                    Token::Await => "await".to_string(),
                    Token::Yield => "yield".to_string(),
                    Token::Const => "const".to_string(),
                    Token::Let => "let".to_string(),
                    Token::Var => "var".to_string(),
                    Token::Import => "import".to_string(),
                    Token::Export => "export".to_string(),
                    Token::Default => "default".to_string(),
                    Token::From => "from".to_string(),
                    Token::As => "as".to_string(),
                    Token::Type => "type".to_string(),
                    Token::Interface => "interface".to_string(),
                    Token::Enum => "enum".to_string(),
                    Token::Static => "static".to_string(),
                    Token::Extends => "extends".to_string(),
                    Token::Super => "super".to_string(),
                    Token::Of => "of".to_string(),
                    Token::Constructor => wk::CONSTRUCTOR.to_string(),
                    Token::Promise => wk::PROMISE.to_string(),
                    Token::Try => "try".to_string(),
                    Token::Public => "public".to_string(),
                    Token::Private => "private".to_string(),
                    Token::Protected => "protected".to_string(),
                    Token::Readonly => "readonly".to_string(),
                    token => {
                        return Err(Error::ParseError(format!(
                            "Expected identifier or pattern, got {:?}",
                            token
                        )))
                    }
                };
                Ok(BindingPattern::Identifier(id))
            }
        }
    }

    pub(crate) fn parse_array_binding_pattern(&mut self) -> Result<BindingPattern> {
        self.expect(&Token::LeftBracket)?;
        let mut elements = Vec::new();
        if self.peek().token != Token::RightBracket {
            loop {
                if self.peek().token == Token::Comma {
                    elements.push(ArrayBindingElement::Skip);
                    self.advance();
                    continue;
                }
                if self.peek().token == Token::Ellipsis {
                    self.advance();
                    let rest = self.parse_binding_pattern()?;
                    elements.push(ArrayBindingElement::Rest(Box::new(rest)));
                    break;
                }
                let pattern = self.parse_binding_pattern()?;
                let default = if self.peek().token == Token::Assign {
                    self.advance();
                    Some(self.parse_expression()?.inner)
                } else {
                    None
                };
                elements.push(ArrayBindingElement::Pattern(pattern, Box::new(default)));
                if self.peek().token == Token::Comma {
                    self.advance();
                    if self.peek().token == Token::RightBracket {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::RightBracket)?;
        Ok(BindingPattern::Array(elements))
    }

    pub(crate) fn parse_object_binding_pattern(&mut self) -> Result<BindingPattern> {
        self.expect(&Token::LeftBrace)?;
        let mut elements = Vec::new();
        if self.peek().token != Token::RightBrace {
            loop {
                if self.peek().token == Token::Ellipsis {
                    self.advance();
                    let rest = self.parse_binding_pattern()?;
                    elements.push(ObjectBindingElement {
                        key: match &rest {
                            BindingPattern::Identifier(name) => name.clone(),
                            _ => {
                                return Err(Error::ParseError(
                                    "Invalid rest pattern in object".into(),
                                ))
                            }
                        },
                        value: rest,
                        shorthand: true,
                        default_value: None,
                        is_rest: true,
                    });
                    break;
                }
                let key_expr = self.token_to_property_name()?;
                let key = match key_expr {
                    Expression::Identifier(name) => name,
                    _ => return Err(Error::ParseError("Expected property name".into())),
                };
                if self.peek().token == Token::Colon {
                    self.advance();
                    let value = self.parse_binding_pattern()?;
                    let default = if self.peek().token == Token::Assign {
                        self.advance();
                        Some(self.parse_expression()?.inner)
                    } else {
                        None
                    };
                    elements.push(ObjectBindingElement {
                        key: key.clone(),
                        value,
                        shorthand: false,
                        default_value: default,
                        is_rest: false,
                    });
                } else if self.peek().token == Token::Assign {
                    self.advance();
                    let default_value = self.parse_expression()?.inner;
                    elements.push(ObjectBindingElement {
                        key: key.clone(),
                        value: BindingPattern::Identifier(key),
                        shorthand: true,
                        default_value: Some(default_value),
                        is_rest: false,
                    });
                } else {
                    elements.push(ObjectBindingElement {
                        key: key.clone(),
                        value: BindingPattern::Identifier(key),
                        shorthand: true,
                        default_value: None,
                        is_rest: false,
                    });
                }
                if self.peek().token == Token::Comma {
                    self.advance();
                    if self.peek().token == Token::RightBrace {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::RightBrace)?;
        Ok(BindingPattern::Object(elements))
    }
}
