use super::*;
use crate::errors::Result;
use crate::well_known as wk;

impl TypeChecker {
    pub(crate) fn check_expression(&mut self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::NumberLiteral(n) => Ok(Type::NumberLiteral(*n)),
            Expression::BigIntLiteral(_) => Ok(Type::Any),
            Expression::StringLiteral(s) => Ok(Type::StringLiteral(s.clone())),
            Expression::RegexLiteral { .. } => Ok(Type::Any),
            Expression::BooleanLiteral(b) => Ok(Type::BooleanLiteral(*b)),
            Expression::NullLiteral => Ok(Type::Null),
            Expression::UndefinedLiteral => Ok(Type::Undefined),
            Expression::NaNLiteral => Ok(Type::Number),
            Expression::InfinityLiteral => Ok(Type::Number),
            Expression::Identifier(name) => {
                if let Some(ty) = self.narrowed_types.get(name) {
                    return Ok(ty.clone());
                }
                self.get_variable_type(name)
                    .ok_or_else(|| Error::ReferenceError(format!("'{}' is not defined", name)))
            }
            Expression::BinaryOp { op, left, right } => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;
                self.check_binary_op(op, &left_type, &right_type)
            }
            Expression::UnaryOp { op, operand } => {
                let operand_type = self.check_expression(operand)?;
                self.check_unary_op(op, &operand_type)
            }
            Expression::Assignment { target, value, .. } => {
                let target_type = self.check_expression(target)?;
                let value_type = self.check_expression(value)?;
                if !self.is_compatible(&target_type, &value_type) {
                    return Err(Error::TypeError(format!(
                        "Type '{:?}' is not assignable to type '{:?}'",
                        value_type, target_type
                    )));
                }
                Ok(target_type)
            }
            Expression::Call { callee, args } => {
                let callee_type = self.check_expression(callee)?;
                let arg_types: Vec<Type> = args
                    .iter()
                    .map(|a| self.check_expression(a))
                    .collect::<Result<Vec<_>>>()?;
                if let Type::Function {
                    params,
                    return_type,
                } = callee_type
                {
                    for (i, (param_ty, arg_ty)) in params.iter().zip(arg_types.iter()).enumerate() {
                        if !self.is_compatible(param_ty, arg_ty) {
                            return Err(Error::TypeError(format!(
                                "Argument {} type mismatch: expected {:?}, got {:?}",
                                i + 1,
                                param_ty,
                                arg_ty
                            )));
                        }
                    }
                    Ok(*return_type)
                } else {
                    Ok(Type::Any)
                }
            }
            Expression::Member {
                object,
                property,
                computed,
            } => {
                let obj_type = self.check_expression(object)?;
                if *computed {
                    self.check_expression(property)?;
                }
                match &obj_type {
                    Type::Object(props) => {
                        if let Expression::Identifier(name) = property.as_ref() {
                            if !*computed {
                                for prop in props {
                                    if prop.name == *name {
                                        return Ok(prop.ty.clone());
                                    }
                                }
                            }
                        }
                        Ok(Type::Any)
                    }
                    Type::Array(_) => {
                        if let Expression::Identifier(name) = property.as_ref() {
                            if !*computed && name == wk::LENGTH {
                                return Ok(Type::Number);
                            }
                        }
                        Ok(Type::Any)
                    }
                    Type::Named(name) => {
                        if let Some(iface) = self.interfaces.get(name) {
                            if let Expression::Identifier(prop_name) = property.as_ref() {
                                if !*computed {
                                    for prop in &iface.properties {
                                        if prop.name == *prop_name {
                                            return Ok(prop.ty.clone());
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Type::Any)
                    }
                    _ => Ok(Type::Any),
                }
            }
            Expression::FunctionExpression {
                params,
                param_types,
                return_type,
                body,
                ..
            } => {
                let param_tys: Vec<Type> = if let Some(pts) = param_types {
                    pts.iter()
                        .map(|pt| {
                            if let Some(ann) = pt {
                                self.resolve_annotation(ann)
                            } else {
                                Ok(Type::Any)
                            }
                        })
                        .collect::<Result<Vec<_>>>()?
                } else {
                    params.iter().map(|_| Type::Any).collect()
                };
                let ret_ty = if let Some(rt) = return_type {
                    self.resolve_annotation(rt)?
                } else {
                    Type::Any
                };
                self.enter_scope();
                for (param, pty) in params.iter().zip(param_tys.iter()) {
                    self.define_variable(param, pty.clone());
                }
                for stmt in body {
                    self.check_statement(&stmt.inner)?;
                }
                self.exit_scope();
                Ok(Type::Function {
                    params: param_tys,
                    return_type: Box::new(ret_ty),
                })
            }
            Expression::ArrowFunction {
                params,
                body,
                param_types,
                return_type,
                ..
            } => {
                let param_tys: Vec<Type> = if let Some(pts) = param_types {
                    pts.iter()
                        .map(|pt| {
                            if let Some(ann) = pt {
                                self.resolve_annotation(ann)
                            } else {
                                Ok(Type::Any)
                            }
                        })
                        .collect::<Result<Vec<_>>>()?
                } else {
                    params.iter().map(|_| Type::Any).collect()
                };
                let ret_ty = if let Some(rt) = return_type {
                    self.resolve_annotation(rt)?
                } else {
                    Type::Any
                };
                self.enter_scope();
                for (param, pty) in params.iter().zip(param_tys.iter()) {
                    self.define_variable(param, pty.clone());
                }
                match body.as_ref() {
                    crate::compiler::parser::ArrowFunctionBody::Expression(expr) => {
                        self.check_expression(expr)?;
                    }
                    crate::compiler::parser::ArrowFunctionBody::Block(stmts) => {
                        for stmt in stmts {
                            self.check_statement(&stmt.inner)?;
                        }
                    }
                }
                self.exit_scope();
                Ok(Type::Function {
                    params: param_tys,
                    return_type: Box::new(ret_ty),
                })
            }
            Expression::ArrayLiteral { elements } => {
                if elements.is_empty() {
                    return Ok(Type::Array(Box::new(Type::Any)));
                }
                let elem_types: Vec<Type> = elements
                    .iter()
                    .map(|e| self.check_expression(e))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Type::Tuple(elem_types))
            }
            Expression::ObjectLiteral { properties } => {
                let mut obj_props = Vec::new();
                for prop in properties {
                    if prop.key.is_empty() {
                        continue;
                    }
                    let val_type = self.check_expression(&prop.value)?;
                    obj_props.push(ObjectProperty {
                        name: prop.key.clone(),
                        ty: val_type,
                        optional: false,
                    });
                }
                Ok(Type::Object(obj_props))
            }
            Expression::SpreadElement { argument } => {
                self.check_expression(argument)?;
                Ok(Type::Any)
            }
            Expression::TypeAssertion {
                expression,
                type_annotation,
            } => {
                self.check_expression(expression)?;
                self.resolve_annotation(type_annotation)
            }
            Expression::MetaProperty { .. } => Ok(Type::Any),
            Expression::ConditionalExpression {
                test,
                consequent,
                alternate,
            } => {
                self.check_expression(test)?;
                let cons_type = self.check_expression(consequent)?;
                let alt_type = self.check_expression(alternate)?;
                if self.is_compatible(&cons_type, &alt_type) {
                    Ok(cons_type)
                } else {
                    Ok(Type::Union(vec![cons_type, alt_type]))
                }
            }
            Expression::TemplateLiteral { expressions, .. } => {
                for expr in expressions {
                    self.check_expression(expr)?;
                }
                Ok(Type::String)
            }
            Expression::TaggedTemplate {
                tag, expressions, ..
            } => {
                self.check_expression(tag)?;
                for expr in expressions {
                    self.check_expression(expr)?;
                }
                Ok(Type::Any)
            }
            Expression::NewExpression { callee, args } => {
                self.check_expression(callee)?;
                for arg in args {
                    self.check_expression(arg)?;
                }
                Ok(Type::Any)
            }
            Expression::UpdateExpression { operand, .. } => {
                let _ty = self.check_expression(operand)?;
                Ok(Type::Number)
            }
            Expression::AwaitExpression { argument } => {
                self.check_expression(argument)?;
                Ok(Type::Any)
            }
            Expression::SuperCall { args } => {
                for arg in args {
                    self.check_expression(arg)?;
                }
                Ok(Type::Any)
            }
            Expression::SuperMember { .. } => Ok(Type::Any),
            Expression::ClassExpression {
                name,
                superclass: _,
                body,
            } => {
                if let Some(n) = name {
                    self.define_variable(n, Type::Any);
                }
                self.enter_scope();
                for member in body {
                    if let crate::compiler::parser::ClassMember::Method { params, body, .. } =
                        member
                    {
                        self.enter_scope();
                        for param in params {
                            self.define_variable(param, Type::Any);
                        }
                        for stmt in body {
                            self.check_statement(&stmt.inner)?;
                        }
                        self.exit_scope();
                    }
                }
                self.exit_scope();
                Ok(Type::Any)
            }
            Expression::RestElement { .. } => Ok(Type::Any),
            Expression::OptionalMember { .. } | Expression::OptionalCall { .. } => Ok(Type::Any),
            Expression::ImportExpression { .. } => Ok(Type::Any),
        }
    }
}
