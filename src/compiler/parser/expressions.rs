use super::*;
use crate::errors::{Error, Result};

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
        match self.peek().token.clone() {
            Token::Assign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: None,
                }))
            }
            Token::PlusAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: Some(CompoundAssignmentOp::AddAssign),
                }))
            }
            Token::MinusAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: Some(CompoundAssignmentOp::SubAssign),
                }))
            }
            Token::StarAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: Some(CompoundAssignmentOp::MulAssign),
                }))
            }
            Token::SlashAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: Some(CompoundAssignmentOp::DivAssign),
                }))
            }
            Token::PercentAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: Some(CompoundAssignmentOp::ModAssign),
                }))
            }
            Token::AndAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: Some(CompoundAssignmentOp::AndAssign),
                }))
            }
            Token::OrAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: Some(CompoundAssignmentOp::OrAssign),
                }))
            }
            Token::XorAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: Some(CompoundAssignmentOp::XorAssign),
                }))
            }
            Token::BitAndAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: Some(CompoundAssignmentOp::BitAndAssign),
                }))
            }
            Token::BitOrAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(self.spanned(Expression::Assignment {
                    target: Box::new(left.inner),
                    value: Box::new(value.inner),
                    op: Some(CompoundAssignmentOp::BitOrAssign),
                }))
            }
            _ => Ok(left),
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
        let mut left = self.parse_and()?;
        while self.peek().token == Token::Or {
            self.advance();
            let right = self.parse_and()?;
            left = self.spanned(Expression::BinaryOp {
                op: BinaryOperator::Or,
                left: Box::new(left.inner),
                right: Box::new(right.inner),
            });
        }
        Ok(left)
    }

    fn parse_nullish(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_or()?;
        while self.peek().token == Token::NullishCoalescing {
            self.advance();
            let right = self.parse_or()?;
            left = self.spanned(Expression::BinaryOp {
                op: BinaryOperator::NullishCoalescing,
                left: Box::new(left.inner),
                right: Box::new(right.inner),
            });
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_equality()?;
        while self.peek().token == Token::And {
            self.advance();
            let right = self.parse_equality()?;
            left = self.spanned(Expression::BinaryOp {
                op: BinaryOperator::And,
                left: Box::new(left.inner),
                right: Box::new(right.inner),
            });
        }
        Ok(left)
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
        let mut left = self.parse_bitwise_xor()?;
        while self.peek().token == Token::BitOr {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            left = self.spanned(Expression::BinaryOp {
                op: BinaryOperator::BitOr,
                left: Box::new(left.inner),
                right: Box::new(right.inner),
            });
        }
        Ok(left)
    }

    fn parse_bitwise_xor(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_bitwise_and()?;
        while self.peek().token == Token::BitXor {
            self.advance();
            let right = self.parse_bitwise_and()?;
            left = self.spanned(Expression::BinaryOp {
                op: BinaryOperator::BitXor,
                left: Box::new(left.inner),
                right: Box::new(right.inner),
            });
        }
        Ok(left)
    }

    fn parse_bitwise_and(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_instanceof()?;
        while self.peek().token == Token::BitAnd {
            self.advance();
            let right = self.parse_instanceof()?;
            left = self.spanned(Expression::BinaryOp {
                op: BinaryOperator::BitAnd,
                left: Box::new(left.inner),
                right: Box::new(right.inner),
            });
        }
        Ok(left)
    }

    fn parse_instanceof(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_in()?;
        while self.peek().token == Token::Instanceof {
            self.advance();
            let right = self.parse_in()?;
            left = self.spanned(Expression::BinaryOp {
                op: BinaryOperator::Instanceof,
                left: Box::new(left.inner),
                right: Box::new(right.inner),
            });
        }
        Ok(left)
    }

    fn parse_in(&mut self) -> Result<SpannedNode<Expression>> {
        let mut left = self.parse_comparison()?;
        while self.peek().token == Token::In {
            self.advance();
            let right = self.parse_comparison()?;
            left = self.spanned(Expression::BinaryOp {
                op: BinaryOperator::In,
                left: Box::new(left.inner),
                right: Box::new(right.inner),
            });
        }
        Ok(left)
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

    fn parse_unary(&mut self) -> Result<SpannedNode<Expression>> {
        match self.peek().token.clone() {
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(self.spanned(Expression::UnaryOp {
                    op: UnaryOperator::Negate,
                    operand: Box::new(operand.inner),
                }))
            }
            Token::Plus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(self.spanned(Expression::UnaryOp {
                    op: UnaryOperator::UnaryPlus,
                    operand: Box::new(operand.inner),
                }))
            }
            Token::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(self.spanned(Expression::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(operand.inner),
                }))
            }
            Token::Typeof => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(self.spanned(Expression::UnaryOp {
                    op: UnaryOperator::Typeof,
                    operand: Box::new(operand.inner),
                }))
            }
            Token::Void => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(self.spanned(Expression::UnaryOp {
                    op: UnaryOperator::Void,
                    operand: Box::new(operand.inner),
                }))
            }
            Token::Delete => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(self.spanned(Expression::UnaryOp {
                    op: UnaryOperator::Delete,
                    operand: Box::new(operand.inner),
                }))
            }
            Token::BitNot => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(self.spanned(Expression::UnaryOp {
                    op: UnaryOperator::BitNot,
                    operand: Box::new(operand.inner),
                }))
            }
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
                    Ok(self.spanned(Expression::Identifier("import".to_string())))
                }
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<SpannedNode<Expression>> {
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

    fn parse_new_expression(&mut self) -> Result<SpannedNode<Expression>> {
        self.expect(&Token::New)?;
        let callee = self.parse_new_target()?;
        // Consume optional generic type arguments: new Map<string, string>()
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
        // Chain member access and calls: new Date().toISOString(), new Foo().bar(), etc.
        loop {
            if self.peek().token == Token::Dot {
                self.advance();
                let property = self.token_to_property_name()?;
                expr = self.spanned(Expression::Member {
                    object: Box::new(expr.inner),
                    property: Box::new(property),
                    computed: false,
                });
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
            } else if self.peek().token == Token::LeftBracket {
                self.advance();
                let property = self.parse_expression()?.inner;
                self.expect(&Token::RightBracket)?;
                expr = self.spanned(Expression::Member {
                    object: Box::new(expr.inner),
                    property: Box::new(property),
                    computed: true,
                });
            } else if self.peek().token == Token::LeftParen {
                self.advance();
                let args = self.parse_args()?;
                self.expect(&Token::RightParen)?;
                expr = self.spanned(Expression::Call {
                    callee: Box::new(expr.inner),
                    args,
                });
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_new_target(&mut self) -> Result<SpannedNode<Expression>> {
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
            Token::LeftParen => {
                // new (expr)() — parenthesized target
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                // Allow member access: new (A.B)()
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

    pub(crate) fn parse_call(&mut self) -> Result<SpannedNode<Expression>> {
        let mut expr = self.parse_primary()?;
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

    fn parse_args(&mut self) -> Result<Vec<Expression>> {
        let mut args = Vec::new();
        if self.peek().token != Token::RightParen {
            loop {
                if self.peek().token == Token::Ellipsis {
                    self.advance();
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

    fn parse_primary(&mut self) -> Result<SpannedNode<Expression>> {
        match self.peek().token.clone() {
            Token::Number(n) => {
                self.advance();
                Ok(self.spanned(Expression::NumberLiteral(n)))
            }
            Token::BigInt(ref s) => {
                let s = s.clone();
                self.advance();
                Ok(self.spanned(Expression::BigIntLiteral(s)))
            }
            Token::String(s) => {
                self.advance();
                Ok(self.spanned(Expression::StringLiteral(s)))
            }
            Token::Regex(s) => {
                self.advance();
                let (pattern, flags) = match s.rfind('/') {
                    Some(pos) => (s[..pos].to_string(), s[pos + 1..].to_string()),
                    None => (s.clone(), String::new()),
                };
                Ok(self.spanned(Expression::RegexLiteral { pattern, flags }))
            }
            Token::TemplateLiteral(parts) => {
                self.advance();
                self.parse_template_literal(parts)
            }
            Token::Identifier(name) => {
                self.advance();
                match name.as_str() {
                    "true" => Ok(self.spanned(Expression::BooleanLiteral(true))),
                    "false" => Ok(self.spanned(Expression::BooleanLiteral(false))),
                    "null" => Ok(self.spanned(Expression::NullLiteral)),
                    "undefined" => Ok(self.spanned(Expression::UndefinedLiteral)),
                    "NaN" => Ok(self.spanned(Expression::NaNLiteral)),
                    _ => {
                        if self.peek().token == Token::Arrow {
                            self.advance();
                            self.parse_arrow_body(vec![name], None, vec![], None, None, false)
                        } else {
                            Ok(self.spanned(Expression::Identifier(name)))
                        }
                    }
                }
            }
            Token::LeftParen => {
                self.advance();
                if self.peek().token == Token::RightParen {
                    self.advance();
                    if self.peek().token == Token::Arrow {
                        self.advance();
                        return self.parse_arrow_body(vec![], None, vec![], None, None, false);
                    }
                    return Err(Error::ParseError("Unexpected )".into()));
                }
                if matches!(self.peek().token, Token::Identifier(_) | Token::Ellipsis) {
                    let saved = self.pos;
                    let (params, param_types, defaults, rest_param) = self.parse_typed_params()?;
                    if self.peek().token == Token::RightParen {
                        self.advance();
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
                            );
                        }
                    }
                    self.pos = saved;
                }
                let expr = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                if self.peek().token == Token::Arrow {
                    let params = match &expr.inner {
                        Expression::Identifier(name) => vec![name.clone()],
                        Expression::ArrayLiteral { elements } => {
                            // Destructured array params: ([a, b]) => ...
                            elements
                                .iter()
                                .map(|e| match e {
                                    Expression::Identifier(n) => n.clone(),
                                    _ => format!("__destr_{}", 0),
                                })
                                .collect()
                        }
                        Expression::ObjectLiteral { properties } => {
                            // Destructured object params: ({a, b}) => ...
                            properties.iter().map(|p| p.key.clone()).collect()
                        }
                        _ => {
                            return Err(Error::ParseError(
                                "Invalid arrow function parameter".into(),
                            ))
                        }
                    };
                    self.advance();
                    return self.parse_arrow_body(params, None, vec![], None, None, false);
                }
                Ok(expr)
            }
            Token::Function => {
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
                let (params, param_types, defaults, rest_param) = self.parse_typed_params()?;
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
                    param_types: Some(param_types),
                    defaults,
                    rest_param,
                    return_type,
                    body,
                    is_async: false,
                    is_generator,
                }))
            }
            Token::Async => {
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
                    let (params, param_types, defaults, rest_param) = self.parse_typed_params()?;
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
                    let (params, param_types, defaults, rest_param) = self.parse_typed_params()?;
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
                        )
                    } else {
                        Err(Error::ParseError(
                            "Expected '=>' after async parameters".into(),
                        ))
                    }
                }
            }
            Token::Class => {
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
            Token::Super => {
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
            Token::This => {
                self.advance();
                Ok(self.spanned(Expression::Identifier("this".into())))
            }
            Token::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                if self.peek().token != Token::RightBracket {
                    loop {
                        if self.peek().token == Token::Ellipsis {
                            self.advance();
                            let argument = Box::new(self.parse_expression()?.inner);
                            elements.push(Expression::SpreadElement { argument });
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
            Token::LeftBrace => {
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
                                if self.peek().token == Token::LeftParen {
                                    self.advance();
                                    let (params, param_types, defaults, rest_param) =
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
                                    && matches!(
                                        self.peek().token,
                                        Token::Identifier(_) | Token::String(_)
                                    )
                                {
                                    // Getter or setter: get "prop"() / set "prop"(val)
                                    let prop_name = match self.advance().token {
                                        Token::Identifier(n) => n,
                                        Token::String(s) => s,
                                        _ => unreachable!(),
                                    };
                                    self.expect(&Token::LeftParen)?;
                                    let is_getter = key == "get";
                                    let setter_param = if !is_getter {
                                        // Setter has a parameter
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
                                            params: vec![setter_param
                                                .unwrap_or_else(|| "__set_val".to_string())],
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
            Token::Less => {
                // TypeScript generic arrow function: <T extends Foo>(...) => ...
                // or type assertion <Type>expr
                self.skip_type_parameters();
                // After skipping type params, parse the parenthesized params + arrow body
                if self.peek().token == Token::LeftParen {
                    self.advance();
                    let (params, param_types, defaults, rest_param) = self.parse_typed_params()?;
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
                        );
                    }
                    // If no arrow, it was a type assertion, parse as expression
                    let expr = self.parse_assignment()?;
                    Ok(expr)
                } else {
                    Err(Error::ParseError(
                        "Expected '(' after type parameters in generic arrow function".into(),
                    ))
                }
            }
            // Fallback: treat any keyword token as an identifier in expression position
            // JavaScript allows keywords to be used as values, property names, etc.
            token => {
                let name = match &token {
                    Token::Set => Some("set"),
                    Token::Get => Some("get"),
                    Token::Delete => Some("delete"),
                    Token::Typeof => Some("typeof"),
                    Token::Void => Some("void"),
                    Token::New => Some("new"),
                    Token::Return => Some("return"),
                    Token::If => Some("if"),
                    Token::Else => Some("else"),
                    Token::While => Some("while"),
                    Token::For => Some("for"),
                    Token::Do => Some("do"),
                    Token::Switch => Some("switch"),
                    Token::Case => Some("case"),
                    Token::Break => Some("break"),
                    Token::Continue => Some("continue"),
                    Token::Try => Some("try"),
                    Token::Catch => Some("catch"),
                    Token::Finally => Some("finally"),
                    Token::Throw => Some("throw"),
                    Token::Const => Some("const"),
                    Token::Let => Some("let"),
                    Token::Var => Some("var"),
                    Token::In => Some("in"),
                    Token::Of => Some("of"),
                    Token::Instanceof => Some("instanceof"),
                    Token::Extends => Some("extends"),
                    Token::Static => Some("static"),
                    Token::Public => Some("public"),
                    Token::Private => Some("private"),
                    Token::Protected => Some("protected"),
                    Token::Enum => Some("enum"),
                    Token::Interface => Some("interface"),
                    Token::Yield => Some("yield"),
                    Token::Await => Some("await"),
                    Token::Constructor => Some("constructor"),
                    Token::From => Some("from"),
                    Token::As => Some("as"),
                    Token::Default => Some("default"),
                    Token::Import => Some("import"),
                    Token::Export => Some("export"),
                    Token::Function => Some("function"),
                    Token::Class => Some("class"),
                    Token::Super => Some("super"),
                    _ => None,
                };
                if let Some(name) = name {
                    self.advance();
                    Ok(self.spanned(Expression::Identifier(name.to_string())))
                } else {
                    Err(Error::ParseError(format!("Unexpected token {:?}", token)))
                }
            }
        }
    }

    pub(crate) fn parse_template_literal(
        &mut self,
        parts: Vec<TemplatePart>,
    ) -> Result<SpannedNode<Expression>> {
        let mut quasis = Vec::new();
        let mut expressions = Vec::new();
        let mut text_buf = String::new();
        for part in parts {
            match part {
                TemplatePart::Text(t) => text_buf.push_str(&t),
                TemplatePart::Expression(expr_tokens) => {
                    quasis.push(text_buf.clone());
                    text_buf.clear();
                    let mut owned_tokens = expr_tokens.clone();
                    let mut sub_parser = Parser::new(&mut owned_tokens);
                    let expr = sub_parser.parse_expression()?;
                    expressions.push(expr.inner);
                }
            }
        }
        quasis.push(text_buf);
        Ok(self.spanned(Expression::TemplateLiteral {
            quasis,
            expressions,
        }))
    }

    pub(crate) fn parse_arrow_body(
        &mut self,
        params: Vec<String>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        defaults: Vec<Option<Expression>>,
        rest_param: Option<String>,
        return_type: Option<TypeAnnotation>,
        is_async: bool,
    ) -> Result<SpannedNode<Expression>> {
        if self.peek().token == Token::LeftBrace {
            self.advance();
            let body = self.parse_block_body()?;
            self.expect(&Token::RightBrace)?;
            Ok(self.spanned(Expression::ArrowFunction {
                params,
                param_types,
                defaults,
                rest_param,
                return_type,
                body: Box::new(ArrowFunctionBody::Block(body)),
                is_async,
            }))
        } else {
            let expr = self.parse_assignment()?;
            Ok(self.spanned(Expression::ArrowFunction {
                params,
                param_types,
                defaults,
                rest_param,
                return_type,
                body: Box::new(ArrowFunctionBody::Expression(expr.inner)),
                is_async,
            }))
        }
    }
}
