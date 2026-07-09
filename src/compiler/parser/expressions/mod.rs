mod arrow;
mod literals;
mod member;
mod primary;
mod unary;

use super::*;
use crate::errors::Result;

macro_rules! parse_left_assoc {
    ($self:ident, $next:ident, $token:pat, $op:expr) => {{
        let mut left = $self.$next()?;
        while matches!($self.peek().token, $token) {
            $self.advance();
            let right = $self.$next()?;
            left = $self.spanned(Expression::BinaryOp {
                op: $op,
                left: Box::new(left.inner),
                right: Box::new(right.inner),
            });
        }
        Ok(left)
    }};
}

impl<'a> Parser<'a> {
    pub(crate) fn parse_expression(&mut self) -> Result<SpannedNode<Expression>> {
        self.parse_assignment()
    }

    pub(crate) fn parse_expression_with_comma(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_assignment()?;
        while self.peek().token == Token::Comma {
            self.advance();
            let right = self.parse_assignment()?;
            left = self.spanned(Expression::BinaryOp {
                op: BinaryOperator::Comma,
                left: Box::new(left.inner),
                right: Box::new(right.inner),
            });
        }
        Ok(left)
    }

    pub(crate) fn parse_assignment(&mut self) -> Result<SpannedNode<Expression>> {
        let left = self.parse_ternary()?;
        let op = match self.peek().token {
            Token::Assign => {
                self.advance();
                let value = self.parse_assignment()?;
                return Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: None,
                }));
            }
            Token::PlusAssign => Some(CompoundAssignmentOp::AddAssign),
            Token::MinusAssign => Some(CompoundAssignmentOp::SubAssign),
            Token::StarAssign => Some(CompoundAssignmentOp::MulAssign),
            Token::SlashAssign => Some(CompoundAssignmentOp::DivAssign),
            Token::PercentAssign => Some(CompoundAssignmentOp::ModAssign),
            Token::AndAssign => Some(CompoundAssignmentOp::AndAssign),
            Token::OrAssign => Some(CompoundAssignmentOp::OrAssign),
            Token::XorAssign => Some(CompoundAssignmentOp::XorAssign),
            Token::BitAndAssign => Some(CompoundAssignmentOp::BitAndAssign),
            Token::BitOrAssign => Some(CompoundAssignmentOp::BitOrAssign),
            Token::ShiftLeftAssign => Some(CompoundAssignmentOp::ShiftLeftAssign),
            Token::ShiftRightAssign => Some(CompoundAssignmentOp::ShiftRightAssign),
            Token::UnsignedShiftRightAssign => Some(CompoundAssignmentOp::UnsignedShiftRightAssign),
            Token::NullishCoalescingAssign => Some(CompoundAssignmentOp::NullishCoalescingAssign),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let value = self.parse_assignment()?;
            Ok(self.spanned(Expression::Assignment {
                target: Box::new(left.inner),
                value: Box::new(value.inner),
                op: Some(op),
            }))
        } else {
            Ok(left)
        }
    }

    fn parse_ternary(&mut self) -> Result<SpannedNode<Expression>> {
        let condition = self.parse_nullish()?;
        if self.peek().token == Token::Question {
            self.advance();
            let consequent = self.parse_assignment()?;
            self.expect(&Token::Colon)?;
            let alternate = self.parse_assignment()?;
            Ok(self.spanned(Expression::ConditionalExpression {
                test: Box::new(condition.inner),
                consequent: Box::new(consequent.inner),
                alternate: Box::new(alternate.inner),
            }))
        } else {
            Ok(condition)
        }
    }

    fn parse_or(&mut self) -> Result<SpannedNode<Expression>> {
        parse_left_assoc!(self, parse_and, Token::Or, BinaryOperator::Or)
    }

    fn parse_nullish(&mut self) -> Result<SpannedNode<Expression>> {
        parse_left_assoc!(
            self,
            parse_or,
            Token::NullishCoalescing,
            BinaryOperator::NullishCoalescing
        )
    }

    fn parse_and(&mut self) -> Result<SpannedNode<Expression>> {
        parse_left_assoc!(self, parse_equality, Token::And, BinaryOperator::And)
    }

    fn parse_equality(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_bitwise_or()?;
        loop {
            let op = match self.peek().token {
                Token::Equal => Some(BinaryOperator::Eq),
                Token::StrictEqual => Some(BinaryOperator::StrictEq),
                Token::NotEqual => Some(BinaryOperator::NotEqual),
                Token::StrictNotEqual => Some(BinaryOperator::StrictNotEqual),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_bitwise_or()?;
                left = self.spanned(Expression::BinaryOp {
                    op,
                    left: Box::new(left.inner),
                    right: Box::new(right.inner),
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_bitwise_or(&mut self) -> Result<SpannedNode<Expression>> {
        parse_left_assoc!(self, parse_bitwise_xor, Token::BitOr, BinaryOperator::BitOr)
    }

    fn parse_bitwise_xor(&mut self) -> Result<SpannedNode<Expression>> {
        parse_left_assoc!(
            self,
            parse_bitwise_and,
            Token::BitXor,
            BinaryOperator::BitXor
        )
    }

    fn parse_bitwise_and(&mut self) -> Result<SpannedNode<Expression>> {
        parse_left_assoc!(
            self,
            parse_instanceof,
            Token::BitAnd,
            BinaryOperator::BitAnd
        )
    }

    fn parse_instanceof(&mut self) -> Result<SpannedNode<Expression>> {
        parse_left_assoc!(
            self,
            parse_in,
            Token::Instanceof,
            BinaryOperator::Instanceof
        )
    }

    fn parse_in(&mut self) -> Result<SpannedNode<Expression>> {
        parse_left_assoc!(self, parse_comparison, Token::In, BinaryOperator::In)
    }

    fn parse_comparison(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_shift()?;
        loop {
            let op = match self.peek().token {
                Token::Less => Some(BinaryOperator::Less),
                Token::Greater => Some(BinaryOperator::Greater),
                Token::LessEqual => Some(BinaryOperator::LessEqual),
                Token::GreaterEqual => Some(BinaryOperator::GreaterEqual),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_shift()?;
                left = self.spanned(Expression::BinaryOp {
                    op,
                    left: Box::new(left.inner),
                    right: Box::new(right.inner),
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek().token {
                Token::ShiftLeft => Some(BinaryOperator::ShiftLeft),
                Token::ShiftRight => Some(BinaryOperator::ShiftRight),
                Token::UnsignedShiftRight => Some(BinaryOperator::UnsignedShiftRight),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_additive()?;
                left = self.spanned(Expression::BinaryOp {
                    op,
                    left: Box::new(left.inner),
                    right: Box::new(right.inner),
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek().token {
                Token::Plus => Some(BinaryOperator::Add),
                Token::Minus => Some(BinaryOperator::Sub),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_multiplicative()?;
                left = self.spanned(Expression::BinaryOp {
                    op,
                    left: Box::new(left.inner),
                    right: Box::new(right.inner),
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_power()?;
        loop {
            let op = match self.peek().token {
                Token::Star => Some(BinaryOperator::Mul),
                Token::Slash => Some(BinaryOperator::Div),
                Token::Percent => Some(BinaryOperator::Mod),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_power()?;
                left = self.spanned(Expression::BinaryOp {
                    op,
                    left: Box::new(left.inner),
                    right: Box::new(right.inner),
                });
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<SpannedNode<Expression>> {
        let left = self.parse_unary()?;
        if self.peek().token == Token::Power {
            self.advance();
            let right = self.parse_unary()?;
            Ok(self.spanned(Expression::BinaryOp {
                op: BinaryOperator::Power,
                left: Box::new(left.inner),
                right: Box::new(right.inner),
            }))
        } else {
            Ok(left)
        }
    }

    fn parse_primary(&mut self) -> Result<SpannedNode<Expression>> {
        match self.peek().token.clone() {
            Token::Number(_)
            | Token::BigInt(_)
            | Token::String(_)
            | Token::Regex(_)
            | Token::TemplateLiteral(_) => self.parse_literal(),
            Token::Identifier(_) => self.parse_identifier_or_keyword(),
            Token::LeftParen => self.parse_paren_or_arrow(),
            Token::Function => self.parse_function_expression(),
            Token::Async => self.parse_async_expression(),
            Token::Class => self.parse_class_expression(),
            Token::Super => self.parse_super_expression(),
            Token::This => self.parse_this_expression(),
            Token::LeftBracket => self.parse_array_literal(),
            Token::LeftBrace => self.parse_object_literal(),
            Token::Less => self.parse_generic_arrow_or_assertion(),
            _ => self.parse_keyword_as_identifier(),
        }
    }
}
