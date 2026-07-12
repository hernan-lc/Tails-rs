use super::super::*;
use crate::errors::Result;

impl<'a> Parser<'a> {
    pub(crate) fn parse_member_chain(
        &mut self,
        mut expr: SpannedNode<Expression>,
    ) -> Result<SpannedNode<Expression>> {
        loop {
            if self.peek().token == Token::LeftParen {
                self.advance();
                let args = self.parse_args()?;
                self.expect(&Token::RightParen)?;
                if matches!(expr.inner, Expression::OptionalMember { .. }) {
                    expr = self.spanned(Expression::OptionalCall {
                        callee: Box::new(expr.inner),
                        args,
                    });
                } else {
                    expr = self.spanned(Expression::Call {
                        callee: Box::new(expr.inner),
                        args,
                    });
                }
            } else if self.peek().token == Token::QuestionDot {
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
            } else if self.peek().token == Token::Dot {
                self.advance();
                let property = self.token_to_property_name()?;
                expr = self.spanned(Expression::Member {
                    object: Box::new(expr.inner),
                    property: Box::new(property),
                    computed: false,
                });
            } else if self.peek().token == Token::LeftBracket {
                self.advance();
                let property = self.parse_expression()?.inner;
                self.expect(&Token::RightBracket)?;
                expr = self.spanned(Expression::Member {
                    object: Box::new(expr.inner),
                    property: Box::new(property),
                    computed: true,
                });
            } else {
                break;
            }
        }
        Ok(expr)
    }

    pub(crate) fn parse_call(&mut self) -> Result<SpannedNode<Expression>> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.peek().token == Token::Not {
                self.advance();
                continue;
            }
            let next = self.peek().token.clone();
            if !matches!(
                next,
                Token::LeftParen | Token::QuestionDot | Token::Dot | Token::LeftBracket
            ) {
                break;
            }
            expr = self.parse_member_chain(expr)?;
        }
        Ok(expr)
    }

    pub(crate) fn parse_args(&mut self) -> Result<Vec<Expression>> {
        let mut args = Vec::new();
        if self.peek().token != Token::RightParen {
            loop {
                if self.peek().token == Token::Ellipsis {
                    self.advance();
                    // Args are AssignmentExpressions (ES): comma separates args,
                    // it is not the comma operator. Use `(a, b)` for comma-expr args.
                    let argument = Box::new(self.parse_assignment()?.inner);
                    args.push(Expression::SpreadElement { argument });
                } else {
                    args.push(self.parse_assignment()?.inner);
                }
                if self.peek().token == Token::Comma {
                    self.advance();
                    if self.peek().token == Token::RightParen {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        Ok(args)
    }
}
