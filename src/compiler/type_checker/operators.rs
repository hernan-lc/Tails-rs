use super::*;
use crate::errors::Result;

impl TypeChecker {
    pub(crate) fn check_binary_op(
        &self,
        op: &BinaryOperator,
        left: &Type,
        right: &Type,
    ) -> Result<Type> {
        // If either type is Any, allow the operation and return Any
        if matches!(left, Type::Any) || matches!(right, Type::Any) {
            return Ok(Type::Any);
        }
        match op {
            BinaryOperator::Add
            | BinaryOperator::Sub
            | BinaryOperator::Mul
            | BinaryOperator::Div
            | BinaryOperator::Mod => {
                if self.is_number_type(left) && self.is_number_type(right) {
                    Ok(Type::Number)
                } else if matches!(op, BinaryOperator::Add)
                    && self.is_string_type(left)
                    && self.is_string_type(right)
                {
                    Ok(Type::String)
                } else if matches!(op, BinaryOperator::Add)
                    && (self.is_number_type(left) || self.is_string_type(left))
                    && (self.is_number_type(right) || self.is_string_type(right))
                {
                    Ok(Type::Any)
                } else {
                    Err(Error::TypeError(format!(
                        "Operator '{:?}' cannot be applied to types '{:?}' and '{:?}'",
                        op, left, right
                    )))
                }
            }
            BinaryOperator::Eq
            | BinaryOperator::StrictEq
            | BinaryOperator::NotEqual
            | BinaryOperator::StrictNotEqual
            | BinaryOperator::Less
            | BinaryOperator::Greater
            | BinaryOperator::LessEqual
            | BinaryOperator::GreaterEqual => Ok(Type::Boolean),
            BinaryOperator::And | BinaryOperator::Or => {
                if matches!(left, Type::Boolean) || matches!(right, Type::Boolean) {
                    Ok(Type::Boolean)
                } else {
                    Ok(Type::Any)
                }
            }
            BinaryOperator::Power => {
                if self.is_number_type(left) && self.is_number_type(right) {
                    Ok(Type::Number)
                } else {
                    Err(Error::TypeError(format!(
                        "Operator '{:?}' cannot be applied to types '{:?}' and '{:?}'",
                        op, left, right
                    )))
                }
            }
            BinaryOperator::Instanceof | BinaryOperator::In => Ok(Type::Boolean),
            BinaryOperator::BitAnd
            | BinaryOperator::BitOr
            | BinaryOperator::BitXor
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight => Ok(Type::Number),
            BinaryOperator::NullishCoalescing => Ok(Type::Any),
        }
    }

    pub(crate) fn check_unary_op(
        &self,
        op: &crate::compiler::parser::UnaryOperator,
        operand: &Type,
    ) -> Result<Type> {
        match op {
            crate::compiler::parser::UnaryOperator::Negate => {
                if self.is_number_type(operand) {
                    Ok(Type::Number)
                } else {
                    Err(Error::TypeError(format!(
                        "Operator '-' cannot be applied to type '{:?}'",
                        operand
                    )))
                }
            }
            crate::compiler::parser::UnaryOperator::Not => Ok(Type::Boolean),
            crate::compiler::parser::UnaryOperator::Typeof => Ok(Type::String),
            crate::compiler::parser::UnaryOperator::Void => Ok(Type::Undefined),
            crate::compiler::parser::UnaryOperator::Delete => Ok(Type::Boolean),
            crate::compiler::parser::UnaryOperator::BitNot => Ok(Type::Number),
        }
    }
}
