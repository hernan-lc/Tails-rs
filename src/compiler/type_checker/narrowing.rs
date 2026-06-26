use super::*;

impl TypeChecker {
    pub(crate) fn apply_type_narrowing(
        &mut self,
        condition: &Expression,
        _cond_type: &Type,
        is_true_branch: bool,
    ) {
        if let Expression::BinaryOp {
            op,
            left,
            right,
        } = condition
        {
            if matches!(op, BinaryOperator::StrictEq) || matches!(op, BinaryOperator::Eq) {
                if let (Expression::UnaryOp { op: crate::compiler::parser::UnaryOperator::Typeof, operand }, Expression::StringLiteral(lit)) =
                    (left.as_ref(), right.as_ref())
                {
                    if let Expression::Identifier(name) = operand.as_ref() {
                        if let Some(orig) = self.get_variable_type(name) {
                            if let Type::Union(variants) = &orig {
                                let narrowed: Vec<Type> = variants
                                    .iter()
                                    .filter(|v| self.type_matches_typeof_value(v, lit))
                                    .cloned()
                                    .collect();
                                if !narrowed.is_empty() {
                                    let ty = if narrowed.len() == 1 {
                                        narrowed[0].clone()
                                    } else {
                                        Type::Union(narrowed)
                                    };
                                    if is_true_branch {
                                        self.narrowed_types.insert(name.clone(), ty);
                                    } else {
                                        let excluded: Vec<Type> = variants
                                            .iter()
                                            .filter(|v| !self.type_matches_typeof_value(v, lit))
                                            .cloned()
                                            .collect();
                                        if !excluded.is_empty() {
                                            let ty = if excluded.len() == 1 {
                                                excluded[0].clone()
                                            } else {
                                                Type::Union(excluded)
                                            };
                                            self.narrowed_types.insert(name.clone(), ty);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if let (Expression::Identifier(name), Expression::StringLiteral(lit)) =
                    (left.as_ref(), right.as_ref())
                {
                    if is_true_branch {
                        self.narrowed_types
                            .insert(name.clone(), Type::StringLiteral(lit.clone()));
                    } else if let Some(orig) = self.get_variable_type(name) {
                        self.narrowed_types.insert(name.clone(), orig);
                    }
                } else if let (Expression::Identifier(name), Expression::NumberLiteral(n)) =
                    (left.as_ref(), right.as_ref())
                {
                    if is_true_branch {
                        self.narrowed_types
                            .insert(name.clone(), Type::NumberLiteral(*n));
                    } else if let Some(orig) = self.get_variable_type(name) {
                        self.narrowed_types.insert(name.clone(), orig);
                    }
                } else if let (Expression::Identifier(name), Expression::BooleanLiteral(b)) =
                    (left.as_ref(), right.as_ref())
                {
                    if is_true_branch {
                        self.narrowed_types
                            .insert(name.clone(), Type::BooleanLiteral(*b));
                    } else if let Some(orig) = self.get_variable_type(name) {
                        self.narrowed_types.insert(name.clone(), orig);
                    }
                } else if let (Expression::Identifier(name), Expression::NullLiteral) =
                    (left.as_ref(), right.as_ref())
                {
                    if is_true_branch {
                        self.narrowed_types
                            .insert(name.clone(), Type::Null);
                    } else if let Some(orig) = self.get_variable_type(name) {
                        self.narrowed_types.insert(name.clone(), orig);
                    }
                }
            } else if matches!(op, BinaryOperator::StrictNotEqual) || matches!(op, BinaryOperator::NotEqual) {
                if let (Expression::Identifier(name), Expression::NullLiteral) =
                    (left.as_ref(), right.as_ref())
                {
                    if is_true_branch {
                        if let Some(orig) = self.get_variable_type(name) {
                            let narrowed = self.remove_null_from_type(&orig);
                            self.narrowed_types.insert(name.clone(), narrowed);
                        }
                    }
                } else if let (Expression::Identifier(name), Expression::UndefinedLiteral) =
                    (left.as_ref(), right.as_ref())
                {
                    if is_true_branch {
                        if let Some(orig) = self.get_variable_type(name) {
                            let narrowed = self.remove_undefined_from_type(&orig);
                            self.narrowed_types.insert(name.clone(), narrowed);
                        }
                    }
                }
            }
        }

        if let Expression::UnaryOp { op, operand } = condition {
            if matches!(op, crate::compiler::parser::UnaryOperator::Typeof) {
                if let Expression::Identifier(name) = operand.as_ref() {
                    if let Some(orig) = self.get_variable_type(name) {
                        if let Type::Union(variants) = &orig {
                            if is_true_branch {
                                let narrowed: Vec<Type> = variants
                                    .iter()
                                    .filter(|v| {
                                        self.type_matches_typeof(v, condition)
                                    })
                                    .cloned()
                                    .collect();
                                if !narrowed.is_empty() {
                                    let ty = if narrowed.len() == 1 {
                                        narrowed[0].clone()
                                    } else {
                                        Type::Union(narrowed)
                                    };
                                    self.narrowed_types.insert(name.clone(), ty);
                                }
                            } else {
                                let narrowed: Vec<Type> = variants
                                    .iter()
                                    .filter(|v| {
                                        !self.type_matches_typeof(v, condition)
                                    })
                                    .cloned()
                                    .collect();
                                if !narrowed.is_empty() {
                                    let ty = if narrowed.len() == 1 {
                                        narrowed[0].clone()
                                    } else {
                                        Type::Union(narrowed)
                                    };
                                    self.narrowed_types.insert(name.clone(), ty);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn type_matches_typeof_value(&self, ty: &Type, typeof_str: &str) -> bool {
        match typeof_str {
            "number" => matches!(ty, Type::Number | Type::NumberLiteral(_)),
            "string" => matches!(ty, Type::String | Type::StringLiteral(_)),
            "boolean" => matches!(ty, Type::Boolean | Type::BooleanLiteral(_)),
            "object" => matches!(ty, Type::Object(_) | Type::Null | Type::Array(_) | Type::Tuple(_)),
            "function" => matches!(ty, Type::Function { .. }),
            "undefined" => matches!(ty, Type::Undefined),
            "bigint" => false,
            _ => true,
        }
    }

    pub(crate) fn type_matches_typeof(&self, ty: &Type, _typeof_expr: &Expression) -> bool {
        match ty {
            Type::Number | Type::NumberLiteral(_) => true,
            Type::String | Type::StringLiteral(_) => true,
            Type::Boolean | Type::BooleanLiteral(_) => true,
            Type::Null => true,
            Type::Undefined => true,
            _ => true,
        }
    }

    pub(crate) fn remove_null_from_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Union(variants) => {
                let filtered: Vec<Type> = variants
                    .iter()
                    .filter(|v| !matches!(v, Type::Null))
                    .cloned()
                    .collect();
                if filtered.len() == 1 {
                    filtered[0].clone()
                } else {
                    Type::Union(filtered)
                }
            }
            Type::Null => Type::Never,
            _ => ty.clone(),
        }
    }

    pub(crate) fn remove_undefined_from_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Union(variants) => {
                let filtered: Vec<Type> = variants
                    .iter()
                    .filter(|v| !matches!(v, Type::Undefined))
                    .cloned()
                    .collect();
                if filtered.len() == 1 {
                    filtered[0].clone()
                } else {
                    Type::Union(filtered)
                }
            }
            Type::Undefined => Type::Never,
            _ => ty.clone(),
        }
    }
}
