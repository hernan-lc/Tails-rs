use super::*;
use crate::errors::{Error, Result};

impl<'a> Parser<'a> {
    pub(crate) fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression> {
        let left = self.parse_ternary()?;
        match self.peek().clone() {
            Token::Assign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment {
                    target: Box::new(left),
                    value: Box::new(value),
                    op: None,
                })
            }
            Token::PlusAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment {
                    target: Box::new(left),
                    value: Box::new(value),
                    op: Some(CompoundAssignmentOp::AddAssign),
                })
            }
            Token::MinusAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment {
                    target: Box::new(left),
                    value: Box::new(value),
                    op: Some(CompoundAssignmentOp::SubAssign),
                })
            }
            Token::StarAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment {
                    target: Box::new(left),
                    value: Box::new(value),
                    op: Some(CompoundAssignmentOp::MulAssign),
                })
            }
            Token::SlashAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment {
                    target: Box::new(left),
                    value: Box::new(value),
                    op: Some(CompoundAssignmentOp::DivAssign),
                })
            }
            Token::PercentAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment {
                    target: Box::new(left),
                    value: Box::new(value),
                    op: Some(CompoundAssignmentOp::ModAssign),
                })
            }
            Token::AndAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment {
                    target: Box::new(left),
                    value: Box::new(value),
                    op: Some(CompoundAssignmentOp::AndAssign),
                })
            }
            Token::OrAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment {
                    target: Box::new(left),
                    value: Box::new(value),
                    op: Some(CompoundAssignmentOp::OrAssign),
                })
            }
            _ => Ok(left),
        }
    }

    fn parse_ternary(&mut self) -> Result<Expression> {
        let condition = self.parse_nullish()?;
        if self.peek() == &Token::Question {
            self.advance();
            let consequent = self.parse_assignment()?;
            self.expect(&Token::Colon)?;
            let alternate = self.parse_assignment()?;
            Ok(Expression::ConditionalExpression {
                test: Box::new(condition),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
            })
        } else {
            Ok(condition)
        }
    }

    fn parse_or(&mut self) -> Result<Expression> {
        let mut left = self.parse_and()?;
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.parse_and()?;
            left = Expression::BinaryOp {
                op: BinaryOperator::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_nullish(&mut self) -> Result<Expression> {
        let mut left = self.parse_or()?;
        while self.peek() == &Token::NullishCoalescing {
            self.advance();
            let right = self.parse_or()?;
            left = Expression::BinaryOp {
                op: BinaryOperator::NullishCoalescing,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression> {
        let mut left = self.parse_equality()?;
        while self.peek() == &Token::And {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::BinaryOp {
                op: BinaryOperator::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression> {
        let mut left = self.parse_instanceof()?;
        loop {
            let op = match self.peek() {
                Token::Equal => Some(BinaryOperator::Eq),
                Token::StrictEqual => Some(BinaryOperator::StrictEq),
                Token::NotEqual => Some(BinaryOperator::NotEqual),
                Token::StrictNotEqual => Some(BinaryOperator::StrictNotEqual),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_instanceof()?;
                left = Expression::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_instanceof(&mut self) -> Result<Expression> {
        let mut left = self.parse_in()?;
        while self.peek() == &Token::Instanceof {
            self.advance();
            let right = self.parse_in()?;
            left = Expression::BinaryOp {
                op: BinaryOperator::Instanceof,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_in(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison()?;
        while self.peek() == &Token::In {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::BinaryOp {
                op: BinaryOperator::In,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut left = self.parse_shift()?;
        loop {
            let op = match self.peek() {
                Token::Less => Some(BinaryOperator::Less),
                Token::Greater => Some(BinaryOperator::Greater),
                Token::LessEqual => Some(BinaryOperator::LessEqual),
                Token::GreaterEqual => Some(BinaryOperator::GreaterEqual),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_shift()?;
                left = Expression::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expression> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Token::ShiftLeft => Some(BinaryOperator::ShiftLeft),
                Token::ShiftRight => Some(BinaryOperator::ShiftRight),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_additive()?;
                left = Expression::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Plus => Some(BinaryOperator::Add),
                Token::Minus => Some(BinaryOperator::Sub),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_multiplicative()?;
                left = Expression::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression> {
        let mut left = self.parse_power()?;
        loop {
            let op = match self.peek() {
                Token::Star => Some(BinaryOperator::Mul),
                Token::Slash => Some(BinaryOperator::Div),
                Token::Percent => Some(BinaryOperator::Mod),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_power()?;
                left = Expression::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expression> {
        let left = self.parse_unary()?;
        if self.peek() == &Token::Power {
            self.advance();
            let right = self.parse_unary()?;
            Ok(Expression::BinaryOp {
                op: BinaryOperator::Power,
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_unary(&mut self) -> Result<Expression> {
        match self.peek().clone() {
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Negate,
                    operand: Box::new(operand),
                })
            }
            Token::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(operand),
                })
            }
            Token::Typeof => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Typeof,
                    operand: Box::new(operand),
                })
            }
            Token::Void => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Void,
                    operand: Box::new(operand),
                })
            }
            Token::Delete => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Delete,
                    operand: Box::new(operand),
                })
            }
            Token::BitNot => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::BitNot,
                    operand: Box::new(operand),
                })
            }
            Token::Increment => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UpdateExpression {
                    op: UpdateOperator::Increment,
                    operand: Box::new(operand),
                    prefix: true,
                })
            }
            Token::Decrement => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UpdateExpression {
                    op: UpdateOperator::Decrement,
                    operand: Box::new(operand),
                    prefix: true,
                })
            }
            Token::New => self.parse_new_expression(),
            Token::Await => {
                self.advance();
                let argument = self.parse_unary()?;
                Ok(Expression::AwaitExpression {
                    argument: Box::new(argument),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression> {
        let mut expr = self.parse_call()?;
        loop {
            match self.peek() {
                Token::Increment => {
                    self.advance();
                    expr = Expression::UpdateExpression {
                        op: UpdateOperator::Increment,
                        operand: Box::new(expr),
                        prefix: false,
                    };
                }
                Token::Decrement => {
                    self.advance();
                    expr = Expression::UpdateExpression {
                        op: UpdateOperator::Decrement,
                        operand: Box::new(expr),
                        prefix: false,
                    };
                }
                Token::As => {
                    self.advance();
                    let type_annotation = self.parse_type_annotation()?;
                    expr = Expression::TypeAssertion {
                        expression: Box::new(expr),
                        type_annotation,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_new_expression(&mut self) -> Result<Expression> {
        self.expect(&Token::New)?;
        let callee = self.parse_new_target()?;
        let args = if self.peek() == &Token::LeftParen {
            self.advance();
            let a = self.parse_args()?;
            self.expect(&Token::RightParen)?;
            a
        } else {
            Vec::new()
        };
        Ok(Expression::NewExpression {
            callee: Box::new(callee),
            args,
        })
    }

    fn parse_new_target(&mut self) -> Result<Expression> {
        match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                let mut expr = Expression::Identifier(name);
                while self.peek() == &Token::Dot {
                    self.advance();
                    let prop_name = self.token_to_property_name()?;
                    expr = Expression::Member {
                        object: Box::new(expr),
                        property: Box::new(prop_name),
                        computed: false,
                    };
                }
                Ok(expr)
            }
            _ => Err(Error::ParseError(format!(
                "Expected identifier after 'new', got {:?}",
                self.peek()
            ))),
        }
    }

    pub(crate) fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.peek() == &Token::LeftParen {
                self.advance();
                let args = self.parse_args()?;
                self.expect(&Token::RightParen)?;
                if matches!(expr, Expression::OptionalMember { .. }) {
                    expr = Expression::OptionalCall {
                        callee: Box::new(expr),
                        args,
                    };
                } else {
                    expr = Expression::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
            } else if self.peek() == &Token::QuestionDot {
                self.advance();
                if self.peek() == &Token::LeftParen {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&Token::RightParen)?;
                    expr = Expression::OptionalCall {
                        callee: Box::new(expr),
                        args,
                    };
                } else if self.peek() == &Token::LeftBracket {
                    self.advance();
                    let property = self.parse_expression()?;
                    self.expect(&Token::RightBracket)?;
                    expr = Expression::OptionalMember {
                        object: Box::new(expr),
                        property: Box::new(property),
                        computed: true,
                    };
                } else {
                    let property = self.token_to_property_name()?;
                    expr = Expression::OptionalMember {
                        object: Box::new(expr),
                        property: Box::new(property),
                        computed: false,
                    };
                }
            } else if self.peek() == &Token::Dot {
                self.advance();
                let property = self.token_to_property_name()?;
                expr = Expression::Member {
                    object: Box::new(expr),
                    property: Box::new(property),
                    computed: false,
                };
            } else if self.peek() == &Token::LeftBracket {
                self.advance();
                let property = self.parse_expression()?;
                self.expect(&Token::RightBracket)?;
                expr = Expression::Member {
                    object: Box::new(expr),
                    property: Box::new(property),
                    computed: true,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expression>> {
        let mut args = Vec::new();
        if self.peek() != &Token::RightParen {
            loop {
                args.push(self.parse_assignment()?);
                if self.peek() == &Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Expression::NumberLiteral(n))
            }
            Token::String(s) => {
                self.advance();
                Ok(Expression::StringLiteral(s))
            }
            Token::TemplateLiteral(parts) => {
                self.advance();
                self.parse_template_literal(parts)
            }
            Token::Identifier(name) => {
                self.advance();
                match name.as_str() {
                    "true" => Ok(Expression::BooleanLiteral(true)),
                    "false" => Ok(Expression::BooleanLiteral(false)),
                    "null" => Ok(Expression::NullLiteral),
                    "undefined" => Ok(Expression::UndefinedLiteral),
                    "NaN" => Ok(Expression::NaNLiteral),
                    _ => {
                        if self.peek() == &Token::Arrow {
                            self.advance();
                            self.parse_arrow_body(vec![name], None, None, false)
                        } else {
                            Ok(Expression::Identifier(name))
                        }
                    }
                }
            }
            Token::LeftParen => {
                self.advance();
                if self.peek() == &Token::RightParen {
                    self.advance();
                    if self.peek() == &Token::Arrow {
                        self.advance();
                        return self.parse_arrow_body(vec![], None, None, false);
                    }
                    return Err(Error::ParseError("Unexpected )".into()));
                }
                if let Token::Identifier(_) = self.peek().clone() {
                    let saved = self.pos;
                    let (params, param_types) = self.parse_typed_params()?;
                    if self.peek() == &Token::RightParen {
                        self.advance();
                        let return_type = if self.peek() == &Token::Colon {
                            self.advance();
                            Some(self.parse_type_annotation()?)
                        } else {
                            None
                        };
                        if self.peek() == &Token::Arrow {
                            self.advance();
                            return self.parse_arrow_body(
                                params,
                                Some(param_types),
                                return_type,
                                false,
                            );
                        }
                    }
                    self.pos = saved;
                }
                let expr = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                if self.peek() == &Token::Arrow {
                    let params = match &expr {
                        Expression::Identifier(name) => vec![name.clone()],
                        _ => {
                            return Err(Error::ParseError(
                                "Invalid arrow function parameter".into(),
                            ))
                        }
                    };
                    self.advance();
                    return self.parse_arrow_body(params, None, None, false);
                }
                Ok(expr)
            }
            Token::Function => {
                self.advance();
                let is_generator = self.peek() == &Token::Star;
                if is_generator {
                    self.advance();
                }
                let name = if let Token::Identifier(_) = self.peek().clone() {
                    match self.advance() {
                        Token::Identifier(n) => Some(n),
                        _ => unreachable!(),
                    }
                } else {
                    None
                };
                self.expect(&Token::LeftParen)?;
                let (params, param_types) = self.parse_typed_params()?;
                self.expect(&Token::RightParen)?;
                let return_type = if self.peek() == &Token::Colon {
                    self.advance();
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_block_body()?;
                self.expect(&Token::RightBrace)?;
                Ok(Expression::FunctionExpression {
                    name,
                    params,
                    param_types: Some(param_types),
                    return_type,
                    body,
                    is_async: false,
                    is_generator,
                })
            }
            Token::Async => {
                self.advance();
                if self.peek() == &Token::Function {
                    self.advance();
                    let is_generator = self.peek() == &Token::Star;
                    if is_generator {
                        self.advance();
                    }
                    let name = if let Token::Identifier(_) = self.peek().clone() {
                        match self.advance() {
                            Token::Identifier(n) => Some(n),
                            _ => unreachable!(),
                        }
                    } else {
                        None
                    };
                    self.expect(&Token::LeftParen)?;
                    let (params, param_types) = self.parse_typed_params()?;
                    self.expect(&Token::RightParen)?;
                    let return_type = if self.peek() == &Token::Colon {
                        self.advance();
                        Some(self.parse_type_annotation()?)
                    } else {
                        None
                    };
                    self.expect(&Token::LeftBrace)?;
                    let body = self.parse_block_body()?;
                    self.expect(&Token::RightBrace)?;
                    Ok(Expression::FunctionExpression {
                        name,
                        params,
                        param_types: Some(param_types),
                        return_type,
                        body,
                        is_async: true,
                        is_generator,
                    })
                } else {
                    self.expect(&Token::LeftParen)?;
                    let (params, param_types) = self.parse_typed_params()?;
                    self.expect(&Token::RightParen)?;
                    let return_type = if self.peek() == &Token::Colon {
                        self.advance();
                        Some(self.parse_type_annotation()?)
                    } else {
                        None
                    };
                    if self.peek() == &Token::Arrow {
                        self.advance();
                        self.parse_arrow_body(params, Some(param_types), return_type, true)
                    } else {
                        Err(Error::ParseError(
                            "Expected '=>' after async parameters".into(),
                        ))
                    }
                }
            }
            Token::Class => {
                self.advance();
                let name = if let Token::Identifier(_) = self.peek().clone() {
                    match self.advance() {
                        Token::Identifier(n) => Some(n),
                        _ => unreachable!(),
                    }
                } else {
                    None
                };
                let superclass = if self.peek() == &Token::Extends {
                    self.advance();
                    Some(self.parse_call()?)
                } else {
                    None
                };
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_class_body()?;
                self.expect(&Token::RightBrace)?;
                Ok(Expression::ClassExpression {
                    name,
                    superclass: superclass.map(Box::new),
                    body,
                })
            }
            Token::Super => {
                self.advance();
                if self.peek() == &Token::LeftParen {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&Token::RightParen)?;
                    Ok(Expression::SuperCall { args })
                } else if self.peek() == &Token::Dot {
                    self.advance();
                    let property = match self.advance() {
                        Token::Identifier(name) => Expression::Identifier(name),
                        t => {
                            return Err(Error::ParseError(format!(
                                "Expected property name after 'super', got {:?}",
                                t
                            )))
                        }
                    };
                    Ok(Expression::SuperMember {
                        property: Box::new(property),
                        computed: false,
                    })
                } else if self.peek() == &Token::LeftBracket {
                    self.advance();
                    let property = self.parse_expression()?;
                    self.expect(&Token::RightBracket)?;
                    Ok(Expression::SuperMember {
                        property: Box::new(property),
                        computed: true,
                    })
                } else {
                    Err(Error::ParseError(
                        "Expected '.' or '(' after 'super'".into(),
                    ))
                }
            }
            Token::This => {
                self.advance();
                Ok(Expression::Identifier("this".into()))
            }
            Token::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                if self.peek() != &Token::RightBracket {
                    loop {
                        if self.peek() == &Token::Ellipsis {
                            self.advance();
                            let argument = Box::new(self.parse_expression()?);
                            elements.push(Expression::SpreadElement { argument });
                        } else {
                            elements.push(self.parse_expression()?);
                        }
                        if self.peek() != &Token::Comma {
                            break;
                        }
                        self.advance();
                        if self.peek() == &Token::RightBracket {
                            break;
                        }
                    }
                }
                self.expect(&Token::RightBracket)?;
                Ok(Expression::ArrayLiteral { elements })
            }
            Token::LeftBrace => {
                self.advance();
                let mut properties = Vec::new();
                if self.peek() != &Token::RightBrace {
                    loop {
                        if self.peek() == &Token::Ellipsis {
                            self.advance();
                            let argument = Box::new(self.parse_expression()?);
                            properties.push(ObjectProperty {
                                key: String::new(),
                                value: Expression::SpreadElement { argument },
                                shorthand: false,
                                computed: false,
                                computed_key: None,
                            });
                        } else if self.peek() == &Token::LeftBracket {
                            self.advance();
                            let key_expr = self.parse_expression()?;
                            self.expect(&Token::RightBracket)?;
                            self.expect(&Token::Colon)?;
                            let value = self.parse_expression()?;
                            properties.push(ObjectProperty {
                                key: String::new(),
                                value,
                                shorthand: false,
                                computed: true,
                                computed_key: Some(key_expr),
                            });
                        } else {
                            let key = self.token_to_key_string()?;
                            if self.peek() == &Token::Colon {
                                self.expect(&Token::Colon)?;
                                let value = self.parse_expression()?;
                                properties.push(ObjectProperty {
                                    key: key.clone(),
                                    value,
                                    shorthand: false,
                                    computed: false,
                                    computed_key: None,
                                });
                            } else {
                                properties.push(ObjectProperty {
                                    key: key.clone(),
                                    value: Expression::Identifier(key),
                                    shorthand: true,
                                    computed: false,
                                    computed_key: None,
                                });
                            }
                        }
                        if self.peek() != &Token::Comma {
                            break;
                        }
                        self.advance();
                        if self.peek() == &Token::RightBrace {
                            break;
                        }
                    }
                }
                self.expect(&Token::RightBrace)?;
                Ok(Expression::ObjectLiteral { properties })
            }
            token => Err(Error::ParseError(format!("Unexpected token {:?}", token))),
        }
    }

    pub(crate) fn parse_template_literal(
        &mut self,
        parts: Vec<TemplatePart>,
    ) -> Result<Expression> {
        let mut quasis = Vec::new();
        let mut expressions = Vec::new();
        let mut text_buf = String::new();
        for part in parts {
            match part {
                TemplatePart::Text(t) => text_buf.push_str(&t),
                TemplatePart::Expression(expr_tokens) => {
                    quasis.push(text_buf.clone());
                    text_buf.clear();
                    let mut sub_parser = Parser::new(&expr_tokens);
                    let expr = sub_parser.parse_expression()?;
                    expressions.push(expr);
                }
            }
        }
        quasis.push(text_buf);
        Ok(Expression::TemplateLiteral {
            quasis,
            expressions,
        })
    }

    pub(crate) fn parse_arrow_body(
        &mut self,
        params: Vec<String>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        return_type: Option<TypeAnnotation>,
        is_async: bool,
    ) -> Result<Expression> {
        if self.peek() == &Token::LeftBrace {
            self.advance();
            let body = self.parse_block_body()?;
            self.expect(&Token::RightBrace)?;
            Ok(Expression::ArrowFunction {
                params,
                param_types,
                return_type,
                body: Box::new(ArrowFunctionBody::Block(body)),
                is_async,
            })
        } else {
            let expr = self.parse_assignment()?;
            Ok(Expression::ArrowFunction {
                params,
                param_types,
                return_type,
                body: Box::new(ArrowFunctionBody::Expression(expr)),
                is_async,
            })
        }
    }
}
