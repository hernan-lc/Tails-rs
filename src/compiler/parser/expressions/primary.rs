use super::super::*;
use crate::errors::{Error, Result};

impl<'a> Parser<'a> {
    pub(crate) fn parse_paren_or_arrow(&mut self) -> Result<SpannedNode<Expression>> {
        self.advance();
        if self.peek().token == Token::RightParen {
            self.advance();
            if self.peek().token == Token::Arrow {
                self.advance();
                return self.parse_arrow_body(vec![], None, vec![], None, None, false, vec![]);
            }
            return Err(Error::ParseError("Unexpected )".into()));
        }
        // Speculatively parse as arrow params (including destructuring). On any
        // failure or non-`=>` follow, rewind and parse as a parenthesized expr.
        if matches!(
            self.peek().token,
            Token::Identifier(_) | Token::Ellipsis | Token::LeftBracket | Token::LeftBrace
        ) {
            let saved = self.pos;
            let arrow_result = (|| -> Result<SpannedNode<Expression>> {
                let (params, param_types, defaults, rest_param, param_patterns) =
                    self.parse_typed_params()?;
                self.expect(&Token::RightParen)?;
                let return_type = if self.peek().token == Token::Colon {
                    self.advance();
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };
                if self.peek().token != Token::Arrow {
                    return Err(Error::ParseError("not an arrow".into()));
                }
                self.advance();
                self.parse_arrow_body(
                    params,
                    Some(param_types),
                    defaults,
                    rest_param,
                    return_type,
                    false,
                    param_patterns,
                )
            })();
            match arrow_result {
                Ok(expr) => return Ok(expr),
                Err(_) => {
                    self.pos = saved;
                }
            }
        }
        let expr = self.parse_expression_with_comma()?;
        self.expect(&Token::RightParen)?;
        if self.peek().token == Token::Arrow {
            let params = match &expr.inner {
                Expression::Identifier(name) => vec![name.clone()],
                Expression::ArrayLiteral { elements } => elements
                    .iter()
                    .map(|e| match e {
                        Expression::Identifier(n) => n.clone(),
                        _ => format!("__destr_{}", 0),
                    })
                    .collect(),
                Expression::ObjectLiteral { properties } => {
                    properties.iter().map(|p| p.key.clone()).collect()
                }
                _ => return Err(Error::ParseError("Invalid arrow function parameter".into())),
            };
            self.advance();
            return self.parse_arrow_body(params, None, vec![], None, None, false, vec![]);
        }
        Ok(expr)
    }

    pub(crate) fn parse_function_expression(&mut self) -> Result<SpannedNode<Expression>> {
        self.advance();
        let is_generator = self.peek().token == Token::Star;
        if is_generator {
            self.advance();
        }
        // Named function expressions may use reserved words as the name
        // (e.g. `function set(setting, val)` in Express).
        let name = if let Some(n) = token_keyword_string(&self.peek().token) {
            // Only consume if it's not `(` — optional name.
            if self.peek().token != Token::LeftParen {
                self.advance();
                Some(n)
            } else {
                None
            }
        } else {
            None
        };
        self.expect(&Token::LeftParen)?;
        let (params, param_types, defaults, rest_param, param_patterns) =
            self.parse_typed_params()?;
        self.expect(&Token::RightParen)?;
        let return_type = if self.peek().token == Token::Colon {
            self.advance();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        self.expect(&Token::LeftBrace)?;
        let body = self.parse_block_body()?;
        self.expect(&Token::RightBrace)?;
        Ok(self.spanned(Expression::FunctionExpression {
            name,
            params,
            param_patterns,
            param_types: Some(param_types),
            defaults,
            rest_param,
            return_type,
            body,
            is_async: false,
            is_generator,
        }))
    }

    pub(crate) fn parse_async_expression(&mut self) -> Result<SpannedNode<Expression>> {
        let next_is_function = self
            .tokens
            .get(self.pos + 1)
            .map(|t| t.token == Token::Function)
            .unwrap_or(false);
        let next_is_paren = self
            .tokens
            .get(self.pos + 1)
            .map(|t| t.token == Token::LeftParen)
            .unwrap_or(false);
        if next_is_function || next_is_paren {
            self.advance();
            if self.peek().token == Token::Function {
                self.advance();
                let is_generator = self.peek().token == Token::Star;
                if is_generator {
                    self.advance();
                }
                let name = if let Token::Identifier(_) = self.peek().token.clone() {
                    match self.advance().token {
                        Token::Identifier(n) => Some(n),
                        _ => unreachable!(),
                    }
                } else {
                    None
                };
                self.expect(&Token::LeftParen)?;
                let (params, param_types, defaults, rest_param, param_patterns) =
                    self.parse_typed_params()?;
                self.expect(&Token::RightParen)?;
                let return_type = if self.peek().token == Token::Colon {
                    self.advance();
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_block_body()?;
                self.expect(&Token::RightBrace)?;
                Ok(self.spanned(Expression::FunctionExpression {
                    name,
                    params,
                    param_patterns,
                    param_types: Some(param_types),
                    defaults,
                    rest_param,
                    return_type,
                    body,
                    is_async: true,
                    is_generator,
                }))
            } else {
                self.expect(&Token::LeftParen)?;
                let (params, param_types, defaults, rest_param, param_patterns) =
                    self.parse_typed_params()?;
                self.expect(&Token::RightParen)?;
                let return_type = if self.peek().token == Token::Colon {
                    self.advance();
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };
                if self.peek().token == Token::Arrow {
                    self.advance();
                    self.parse_arrow_body(
                        params,
                        Some(param_types),
                        defaults,
                        rest_param,
                        return_type,
                        true,
                        param_patterns,
                    )
                } else {
                    Err(Error::ParseError(
                        "Expected '=>' after async parameters".into(),
                    ))
                }
            }
        } else {
            self.advance();
            if self.peek().token == Token::Arrow {
                self.advance();
                self.parse_arrow_body(
                    vec!["async".to_string()],
                    None,
                    vec![],
                    None,
                    None,
                    false,
                    vec![],
                )
            } else {
                Ok(self.spanned(Expression::Identifier("async".to_string())))
            }
        }
    }

    pub(crate) fn parse_class_expression(&mut self) -> Result<SpannedNode<Expression>> {
        self.advance();
        let name = if let Token::Identifier(_) = self.peek().token.clone() {
            match self.advance().token {
                Token::Identifier(n) => Some(n),
                _ => unreachable!(),
            }
        } else {
            None
        };
        let superclass = if self.peek().token == Token::Extends {
            self.advance();
            Some(self.parse_call()?.inner)
        } else {
            None
        };
        self.expect(&Token::LeftBrace)?;
        let body = self.parse_class_body()?;
        self.expect(&Token::RightBrace)?;
        Ok(self.spanned(Expression::ClassExpression {
            name,
            superclass: superclass.map(Box::new),
            body,
        }))
    }

    pub(crate) fn parse_super_expression(&mut self) -> Result<SpannedNode<Expression>> {
        self.advance();
        if self.peek().token == Token::LeftParen {
            self.advance();
            let args = self.parse_args()?;
            self.expect(&Token::RightParen)?;
            Ok(self.spanned(Expression::SuperCall { args }))
        } else if self.peek().token == Token::Dot {
            self.advance();
            let property = match self.advance().token {
                Token::Identifier(name) => Expression::Identifier(name),
                t => {
                    return Err(Error::ParseError(format!(
                        "Expected property name after 'super', got {:?}",
                        t
                    )))
                }
            };
            Ok(self.spanned(Expression::SuperMember {
                property: Box::new(property),
                computed: false,
            }))
        } else if self.peek().token == Token::LeftBracket {
            self.advance();
            let property = self.parse_expression()?.inner;
            self.expect(&Token::RightBracket)?;
            Ok(self.spanned(Expression::SuperMember {
                property: Box::new(property),
                computed: true,
            }))
        } else {
            Err(Error::ParseError(
                "Expected '.' or '(' after 'super'".into(),
            ))
        }
    }

    pub(crate) fn parse_this_expression(&mut self) -> Result<SpannedNode<Expression>> {
        self.advance();
        Ok(self.spanned(Expression::Identifier("this".into())))
    }

    pub(crate) fn parse_array_literal(&mut self) -> Result<SpannedNode<Expression>> {
        self.advance();
        let mut elements = Vec::new();
        if self.peek().token != Token::RightBracket {
            loop {
                if self.peek().token == Token::Ellipsis {
                    self.advance();
                    let argument = Box::new(self.parse_expression()?.inner);
                    elements.push(Expression::SpreadElement { argument });
                } else if self.peek().token == Token::Comma {
                    elements.push(Expression::UndefinedLiteral);
                } else if self.peek().token == Token::RightBracket {
                    break;
                } else {
                    elements.push(self.parse_expression()?.inner);
                }
                if self.peek().token != Token::Comma {
                    break;
                }
                self.advance();
                if self.peek().token == Token::RightBracket {
                    break;
                }
            }
        }
        self.expect(&Token::RightBracket)?;
        Ok(self.spanned(Expression::ArrayLiteral { elements }))
    }

    pub(crate) fn parse_object_literal(&mut self) -> Result<SpannedNode<Expression>> {
        self.advance();
        let mut properties = Vec::new();
        if self.peek().token != Token::RightBrace {
            loop {
                if self.peek().token == Token::Ellipsis {
                    self.advance();
                    let argument = Box::new(self.parse_expression()?.inner);
                    properties.push(ObjectProperty {
                        key: String::new(),
                        value: Expression::SpreadElement { argument },
                        shorthand: false,
                        computed: false,
                        computed_key: None,
                        is_getter: false,
                        is_setter: false,
                    });
                } else if self.peek().token == Token::LeftBracket {
                    self.advance();
                    let key_expr = self.parse_expression()?.inner;
                    self.expect(&Token::RightBracket)?;
                    if self.peek().token == Token::LeftParen
                        || self.peek().token == Token::Async
                        || self.peek().token == Token::Star
                    {
                        let mut is_async = false;
                        let mut is_generator = false;
                        if self.peek().token == Token::Async {
                            is_async = true;
                            self.advance();
                        }
                        if self.peek().token == Token::Star {
                            is_generator = true;
                            self.advance();
                        }
                        self.expect(&Token::LeftParen)?;
                        let (params, param_types, defaults, rest_param, param_patterns) =
                            self.parse_typed_params()?;
                        self.expect(&Token::RightParen)?;
                        let return_type = if self.peek().token == Token::Colon {
                            self.advance();
                            Some(self.parse_type_annotation()?)
                        } else {
                            None
                        };
                        self.expect(&Token::LeftBrace)?;
                        let body = self.parse_block_body()?;
                        self.expect(&Token::RightBrace)?;
                        properties.push(ObjectProperty {
                            key: String::new(),
                            value: Expression::FunctionExpression {
                                name: None,
                                params,
                                param_patterns,
                                param_types: Some(param_types),
                                defaults,
                                rest_param,
                                return_type,
                                body,
                                is_async,
                                is_generator,
                            },
                            shorthand: false,
                            computed: true,
                            computed_key: Some(key_expr),
                            is_getter: false,
                            is_setter: false,
                        });
                    } else {
                        self.expect(&Token::Colon)?;
                        let value = self.parse_expression()?.inner;
                        properties.push(ObjectProperty {
                            key: String::new(),
                            value,
                            shorthand: false,
                            computed: true,
                            computed_key: Some(key_expr),
                            is_getter: false,
                            is_setter: false,
                        });
                    }
                } else {
                    let saved = self.pos;
                    let mut is_async = false;
                    let mut is_generator = false;
                    if self.peek().token == Token::Async {
                        is_async = true;
                        self.advance();
                    }
                    if self.peek().token == Token::Star {
                        is_generator = true;
                        self.advance();
                    }
                    if let Ok(key) = self.token_to_key_string() {
                        let is_method_like = self.peek().token == Token::LeftParen
                            || (self.peek().token == Token::This && (key == "get" || key == "set"));
                        if is_method_like {
                            self.advance();
                            let (params, param_types, defaults, rest_param, param_patterns) =
                                self.parse_typed_params()?;
                            self.expect(&Token::RightParen)?;
                            let return_type = if self.peek().token == Token::Colon {
                                self.advance();
                                Some(self.parse_type_annotation()?)
                            } else {
                                None
                            };
                            self.expect(&Token::LeftBrace)?;
                            let body = self.parse_block_body()?;
                            self.expect(&Token::RightBrace)?;
                            let mut full_body = vec![];
                            full_body.extend(body);
                            properties.push(ObjectProperty {
                                key: key.clone(),
                                value: Expression::FunctionExpression {
                                    name: Some(key.clone()),
                                    params,
                                    param_patterns,
                                    param_types: Some(param_types),
                                    defaults,
                                    rest_param,
                                    return_type,
                                    body: full_body,
                                    is_async,
                                    is_generator,
                                },
                                shorthand: false,
                                computed: false,
                                computed_key: None,
                                is_getter: false,
                                is_setter: false,
                            });
                        } else if (key == "get" || key == "set")
                            && matches!(self.peek().token, Token::Identifier(_) | Token::String(_))
                        {
                            let prop_name = match self.advance().token {
                                Token::Identifier(n) => n,
                                Token::String(s) => s,
                                _ => unreachable!(),
                            };
                            self.expect(&Token::LeftParen)?;
                            let is_getter = key == "get";
                            let setter_param = if !is_getter {
                                match self.advance().token {
                                    Token::Identifier(name) => Some(name),
                                    t => {
                                        return Err(Error::ParseError(format!(
                                            "Expected setter parameter, got {:?}",
                                            t
                                        )))
                                    }
                                }
                            } else {
                                None
                            };
                            self.expect(&Token::RightParen)?;
                            let return_type = if self.peek().token == Token::Colon {
                                self.advance();
                                Some(self.parse_type_annotation()?)
                            } else {
                                None
                            };
                            self.expect(&Token::LeftBrace)?;
                            let body = self.parse_block_body()?;
                            self.expect(&Token::RightBrace)?;
                            let accessor_fn = if is_getter {
                                Expression::FunctionExpression {
                                    name: Some(prop_name.clone()),
                                    params: vec![],
                                    param_patterns: vec![],
                                    param_types: Some(vec![]),
                                    defaults: vec![],
                                    rest_param: None,
                                    return_type,
                                    body,
                                    is_async: false,
                                    is_generator: false,
                                }
                            } else {
                                Expression::FunctionExpression {
                                    name: Some(prop_name.clone()),
                                    params: vec![
                                        setter_param.unwrap_or_else(|| "__set_val".to_string())
                                    ],
                                    param_patterns: vec![None],
                                    param_types: Some(vec![None]),
                                    defaults: vec![],
                                    rest_param: None,
                                    return_type,
                                    body,
                                    is_async: false,
                                    is_generator: false,
                                }
                            };
                            properties.push(ObjectProperty {
                                key: prop_name,
                                value: accessor_fn,
                                shorthand: false,
                                computed: false,
                                computed_key: None,
                                is_getter,
                                is_setter: !is_getter,
                            });
                        } else if self.peek().token == Token::Colon {
                            self.expect(&Token::Colon)?;
                            let value = self.parse_expression()?.inner;
                            properties.push(ObjectProperty {
                                key: key.clone(),
                                value,
                                shorthand: false,
                                computed: false,
                                computed_key: None,
                                is_getter: false,
                                is_setter: false,
                            });
                        } else {
                            properties.push(ObjectProperty {
                                key: key.clone(),
                                value: Expression::Identifier(key),
                                shorthand: true,
                                computed: false,
                                computed_key: None,
                                is_getter: false,
                                is_setter: false,
                            });
                        }
                    } else {
                        self.pos = saved;
                        let key = self.token_to_key_string()?;
                        if self.peek().token == Token::Colon {
                            self.expect(&Token::Colon)?;
                            let value = self.parse_expression()?.inner;
                            properties.push(ObjectProperty {
                                key: key.clone(),
                                value,
                                shorthand: false,
                                computed: false,
                                computed_key: None,
                                is_getter: false,
                                is_setter: false,
                            });
                        } else {
                            properties.push(ObjectProperty {
                                key: key.clone(),
                                value: Expression::Identifier(key),
                                shorthand: true,
                                computed: false,
                                computed_key: None,
                                is_getter: false,
                                is_setter: false,
                            });
                        }
                    }
                }
                if self.peek().token != Token::Comma {
                    break;
                }
                self.advance();
                if self.peek().token == Token::RightBrace {
                    break;
                }
            }
        }
        self.expect(&Token::RightBrace)?;
        Ok(self.spanned(Expression::ObjectLiteral { properties }))
    }

    pub(crate) fn parse_generic_arrow_or_assertion(&mut self) -> Result<SpannedNode<Expression>> {
        self.skip_type_parameters();
        if self.peek().token == Token::LeftParen {
            self.advance();
            let (params, param_types, defaults, rest_param, param_patterns) =
                self.parse_typed_params()?;
            self.expect(&Token::RightParen)?;
            let return_type = if self.peek().token == Token::Colon {
                self.advance();
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
            if self.peek().token == Token::Arrow {
                self.advance();
                return self.parse_arrow_body(
                    params,
                    Some(param_types),
                    defaults,
                    rest_param,
                    return_type,
                    false,
                    param_patterns,
                );
            }
            let expr = self.parse_assignment()?;
            Ok(expr)
        } else {
            Err(Error::ParseError(format!(
                "Expected '(' after type parameters in generic arrow function at {}:{}",
                self.current_span.line, self.current_span.col
            )))
        }
    }
}
