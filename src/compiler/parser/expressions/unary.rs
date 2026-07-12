use super::super::*;
use crate::errors::{Error, Result};

impl<'a> Parser<'a> {
    pub(crate) fn parse_unary(&mut self) -> Result<SpannedNode<Expression>> {
        let op = match self.peek().token {
            Token::Minus => Some(UnaryOperator::Negate),
            Token::Plus => Some(UnaryOperator::UnaryPlus),
            Token::Not => Some(UnaryOperator::Not),
            Token::Typeof => Some(UnaryOperator::Typeof),
            Token::Void => Some(UnaryOperator::Void),
            Token::Delete => Some(UnaryOperator::Delete),
            Token::BitNot => Some(UnaryOperator::BitNot),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(self.spanned(Expression::UnaryOp {
                op,
                operand: Box::new(operand.inner),
            }));
        }
        match self.peek().token.clone() {
            Token::Increment => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(self.spanned(Expression::UpdateExpression {
                    op: UpdateOperator::Increment,
                    operand: Box::new(operand.inner),
                    prefix: true,
                }))
            }
            Token::Decrement => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(self.spanned(Expression::UpdateExpression {
                    op: UpdateOperator::Decrement,
                    operand: Box::new(operand.inner),
                    prefix: true,
                }))
            }
            Token::New => self.parse_new_expression(),
            Token::Await => {
                self.advance();
                let argument = self.parse_unary()?;
                Ok(self.spanned(Expression::AwaitExpression {
                    argument: Box::new(argument.inner),
                }))
            }
            Token::Import => {
                self.advance();
                if self.peek().token == Token::LeftParen {
                    self.advance();
                    let source = self.parse_expression()?;
                    self.expect(&Token::RightParen)?;
                    Ok(self.spanned(Expression::ImportExpression {
                        source: Box::new(source.inner),
                    }))
                } else {
                    let mut expr = self.spanned(Expression::Identifier("import".to_string()));
                    loop {
                        match self.peek().token {
                            Token::Dot => {
                                self.advance();
                                let property = self.token_to_property_name()?;
                                expr = self.spanned(Expression::Member {
                                    object: Box::new(expr.inner),
                                    property: Box::new(property),
                                    computed: false,
                                });
                            }
                            Token::LeftBracket => {
                                self.advance();
                                let property = self.parse_expression()?.inner;
                                self.expect(&Token::RightBracket)?;
                                expr = self.spanned(Expression::Member {
                                    object: Box::new(expr.inner),
                                    property: Box::new(property),
                                    computed: true,
                                });
                            }
                            Token::LeftParen => {
                                self.advance();
                                let args = self.parse_args()?;
                                self.expect(&Token::RightParen)?;
                                expr = self.spanned(Expression::Call {
                                    callee: Box::new(expr.inner),
                                    args,
                                });
                            }
                            Token::QuestionDot => {
                                self.advance();
                                if self.peek().token == Token::LeftParen {
                                    self.advance();
                                    let args = self.parse_args()?;
                                    self.expect(&Token::RightParen)?;
                                    expr = self.spanned(Expression::OptionalCall {
                                        callee: Box::new(expr.inner),
                                        args,
                                    });
                                } else if self.peek().token == Token::LeftBracket {
                                    self.advance();
                                    let property = self.parse_expression()?.inner;
                                    self.expect(&Token::RightBracket)?;
                                    expr = self.spanned(Expression::OptionalMember {
                                        object: Box::new(expr.inner),
                                        property: Box::new(property),
                                        computed: true,
                                    });
                                } else {
                                    let property = self.token_to_property_name()?;
                                    expr = self.spanned(Expression::OptionalMember {
                                        object: Box::new(expr.inner),
                                        property: Box::new(property),
                                        computed: false,
                                    });
                                }
                            }
                            _ => break,
                        }
                    }
                    Ok(expr)
                }
            }
            _ => self.parse_postfix(),
        }
    }

    pub(crate) fn parse_postfix(&mut self) -> Result<SpannedNode<Expression>> {
        let mut expr = self.parse_call()?;
        loop {
            match self.peek().token {
                Token::Increment => {
                    self.advance();
                    expr = self.spanned(Expression::UpdateExpression {
                        op: UpdateOperator::Increment,
                        operand: Box::new(expr.inner),
                        prefix: false,
                    });
                }
                Token::Decrement => {
                    self.advance();
                    expr = self.spanned(Expression::UpdateExpression {
                        op: UpdateOperator::Decrement,
                        operand: Box::new(expr.inner),
                        prefix: false,
                    });
                }
                Token::As => {
                    self.advance();
                    let type_annotation = self.parse_type_annotation()?;
                    expr = self.spanned(Expression::TypeAssertion {
                        expression: Box::new(expr.inner),
                        type_annotation,
                    });
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    pub(crate) fn parse_new_expression(&mut self) -> Result<SpannedNode<Expression>> {
        self.expect(&Token::New)?;
        if self.peek().token == Token::Dot {
            self.advance();
            if let Token::Identifier(name) = &self.peek().token {
                if name == "target" {
                    self.advance();
                    return Ok(self.spanned(Expression::MetaProperty {
                        meta: "new".to_string(),
                        property: "target".to_string(),
                    }));
                }
            }
            return Err(Error::ParseError(format!(
                "Unexpected member after 'new', got {:?}",
                self.peek().token
            )));
        }
        let callee = self.parse_new_target()?;
        if self.peek().token == Token::Less {
            self.advance();
            let mut depth = 1;
            while depth > 0 && self.peek().token != Token::Eof {
                match self.peek().token {
                    Token::Less => depth += 1,
                    Token::Greater => depth -= 1,
                    Token::LeftBracket | Token::LeftBrace | Token::LeftParen => {}
                    _ => {}
                }
                self.advance();
            }
        }
        let args = if self.peek().token == Token::LeftParen {
            self.advance();
            let a = self.parse_args()?;
            self.expect(&Token::RightParen)?;
            a
        } else {
            Vec::new()
        };
        let mut expr = self.spanned(Expression::NewExpression {
            callee: Box::new(callee.inner),
            args,
        });
        expr = self.parse_member_chain(expr)?;
        Ok(expr)
    }

    pub(crate) fn parse_new_target(&mut self) -> Result<SpannedNode<Expression>> {
        match self.peek().token.clone() {
            Token::Identifier(name) => {
                self.advance();
                let mut expr = Expression::Identifier(name);
                while self.peek().token == Token::Dot {
                    self.advance();
                    let prop_name = self.token_to_property_name()?;
                    expr = Expression::Member {
                        object: Box::new(expr),
                        property: Box::new(prop_name),
                        computed: false,
                    };
                }
                Ok(self.spanned(expr))
            }
            // `new this(...)` — used by ipaddr.js constructor helpers.
            Token::This => {
                self.advance();
                let mut expr = Expression::Identifier("this".to_string());
                while self.peek().token == Token::Dot {
                    self.advance();
                    let prop_name = self.token_to_property_name()?;
                    expr = Expression::Member {
                        object: Box::new(expr),
                        property: Box::new(prop_name),
                        computed: false,
                    };
                }
                Ok(self.spanned(expr))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                let mut target = expr;
                while self.peek().token == Token::Dot {
                    self.advance();
                    let prop_name = self.token_to_property_name()?;
                    target = self.spanned(Expression::Member {
                        object: Box::new(target.inner),
                        property: Box::new(prop_name),
                        computed: false,
                    });
                }
                Ok(target)
            }
            _ => Err(Error::ParseError(format!(
                "Expected identifier or '(' after 'new', got {:?}",
                self.peek().token
            ))),
        }
    }
}
