use super::*;
use crate::errors::Result;

impl TypeChecker {
    pub(crate) fn check_statement(&mut self, stmt: &Statement) -> Result<Type> {
        match stmt {
            Statement::Expression(expr) => {
                self.check_expression(expr)?;
                Ok(Type::Void)
            }
            Statement::VariableDeclaration { kind, declarations } => {
                for decl in declarations {
                    if let Some(init) = &decl.init {
                        let init_type = self.check_expression(init)?;
                        if let Some(ann) = &decl.type_annotation {
                            let declared = self.resolve_annotation(ann)?;
                            if !self.is_compatible(&declared, &init_type) {
                                return Err(Error::TypeError(format!(
                                    "Type '{:?}' is not assignable to type '{:?}'",
                                    init_type, declared
                                )));
                            }
                            self.define_pattern_variables(&decl.id, &declared);
                        } else {
                            self.define_pattern_variables(&decl.id, &init_type);
                        }
                    } else if matches!(kind, VarKind::Const) {
                        return Err(Error::TypeError(
                            "Missing initializer in const declaration".into(),
                        ));
                    } else if let Some(ann) = &decl.type_annotation {
                        let declared = self.resolve_annotation(ann)?;
                        self.define_pattern_variables(&decl.id, &declared);
                    }
                }
                Ok(Type::Void)
            }
            Statement::ReturnStatement(value) => {
                if let Some(expr) = value {
                    self.check_expression(expr)?;
                }
                Ok(Type::Void)
            }
            Statement::IfStatement {
                condition,
                consequent,
                alternate,
            } => {
                let cond_type = self.check_expression(condition)?;
                let saved = self.narrowed_types.clone();
                self.apply_type_narrowing(condition, &cond_type, true);
                self.check_statement(&consequent.inner)?;
                let after_then = self.narrowed_types.clone();
                self.narrowed_types = saved;
                self.apply_type_narrowing(condition, &cond_type, false);
                if let Some(alt) = alternate {
                    self.check_statement(&alt.inner)?;
                }
                self.narrowed_types = after_then;
                Ok(Type::Void)
            }
            Statement::WhileStatement { condition, body } => {
                self.check_expression(condition)?;
                self.check_statement(&body.inner)?;
                Ok(Type::Void)
            }
            Statement::BlockStatement(stmts) => {
                self.enter_scope();
                for stmt in stmts {
                    self.check_statement(&stmt.inner)?;
                }
                self.exit_scope();
                Ok(Type::Void)
            }
            Statement::FunctionDeclaration {
                name,
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
                let func_type = Type::Function {
                    params: param_tys.clone(),
                    return_type: Box::new(ret_ty),
                };
                self.define_variable(name, func_type);
                self.enter_scope();
                for (param, pty) in params.iter().zip(param_tys.iter()) {
                    self.define_variable(param, pty.clone());
                }
                for stmt in body {
                    self.check_statement(&stmt.inner)?;
                }
                self.exit_scope();
                Ok(Type::Void)
            }
            Statement::InterfaceDeclaration {
                name,
                extends,
                members,
            } => {
                let mut properties = Vec::new();
                for member in members {
                    match member {
                        crate::compiler::parser::InterfaceMember::Property {
                            name,
                            type_annotation,
                            optional,
                        } => {
                            let ty = self.resolve_annotation(type_annotation)?;
                            properties.push(ObjectProperty {
                                name: name.clone(),
                                ty,
                                optional: *optional,
                            });
                        }
                        crate::compiler::parser::InterfaceMember::Method {
                            name,
                            params: _,
                            return_type: _,
                        } => {
                            properties.push(ObjectProperty {
                                name: name.clone(),
                                ty: Type::Any,
                                optional: false,
                            });
                        }
                    }
                }
                self.interfaces.insert(
                    name.clone(),
                    InterfaceDecl {
                        name: name.clone(),
                        extends: extends.clone(),
                        properties,
                    },
                );
                self.define_variable(name, Type::Any);
                Ok(Type::Void)
            }
            Statement::TypeAliasDeclaration {
                name,
                type_annotation,
            } => {
                let ty = self.resolve_annotation(type_annotation)?;
                self.type_aliases.insert(
                    name.clone(),
                    TypeAliasDecl {
                        name: name.clone(),
                        ty: ty.clone(),
                    },
                );
                Ok(Type::Void)
            }
            Statement::EnumDeclaration { name, members } => {
                let mut enum_members = Vec::new();
                let mut value = 0.0;
                for member in members {
                    let member_type = if let Some(val) = &member.value {
                        match val {
                            crate::compiler::parser::TypeLiteral::Number(n) => {
                                value = *n;
                                Type::NumberLiteral(*n)
                            }
                            crate::compiler::parser::TypeLiteral::String(s) => {
                                Type::StringLiteral(s.clone())
                            }
                            crate::compiler::parser::TypeLiteral::Boolean(b) => {
                                Type::BooleanLiteral(*b)
                            }
                        }
                    } else {
                        let t = Type::NumberLiteral(value);
                        value += 1.0;
                        t
                    };
                    enum_members.push((member.name.clone(), member_type));
                }
                self.enums.insert(
                    name.clone(),
                    EnumDecl {
                        name: name.clone(),
                        members: enum_members,
                    },
                );
                self.define_variable(name, Type::Any);
                Ok(Type::Void)
            }
            Statement::ClassDeclaration {
                name,
                superclass: _,
                body,
            } => {
                self.define_variable(name, Type::Any);
                self.enter_scope();
                for member in body {
                    match member {
                        crate::compiler::parser::ClassMember::Method {
                            name: _,
                            params,
                            body,
                            ..
                        } => {
                            self.enter_scope();
                            for param in params {
                                self.define_variable(param, Type::Any);
                            }
                            for stmt in body {
                                self.check_statement(&stmt.inner)?;
                            }
                            self.exit_scope();
                        }
                        crate::compiler::parser::ClassMember::Constructor { params, body } => {
                            self.enter_scope();
                            for param in params {
                                self.define_variable(&param.name, Type::Any);
                            }
                            for stmt in body {
                                self.check_statement(&stmt.inner)?;
                            }
                            self.exit_scope();
                        }
                        _ => {}
                    }
                }
                self.exit_scope();
                Ok(Type::Void)
            }
            Statement::ForStatement {
                init,
                condition,
                update,
                body,
            } => {
                self.enter_scope();
                if let Some(init) = init {
                    match init.as_ref() {
                        crate::compiler::parser::ForInit::Variable(stmt) => {
                            self.check_statement(&stmt.inner)?;
                        }
                        crate::compiler::parser::ForInit::Expression(expr) => {
                            self.check_expression(expr)?;
                        }
                    }
                }
                if let Some(cond) = condition {
                    self.check_expression(cond)?;
                }
                if let Some(upd) = update {
                    self.check_expression(upd)?;
                }
                self.check_statement(&body.inner)?;
                self.exit_scope();
                Ok(Type::Void)
            }
            Statement::ForInStatement {
                left: _,
                right,
                body,
            } => {
                self.check_expression(right)?;
                self.check_statement(&body.inner)?;
                Ok(Type::Void)
            }
            Statement::ForOfStatement {
                left: _,
                right,
                body,
                ..
            } => {
                self.check_expression(right)?;
                self.check_statement(&body.inner)?;
                Ok(Type::Void)
            }
            Statement::SwitchStatement {
                discriminant,
                cases,
            } => {
                self.check_expression(discriminant)?;
                for case in cases {
                    if let Some(test) = &case.test {
                        self.check_expression(test)?;
                    }
                    for stmt in &case.consequent {
                        self.check_statement(&stmt.inner)?;
                    }
                }
                Ok(Type::Void)
            }
            Statement::TryStatement {
                block,
                handler,
                finalizer,
            } => {
                self.enter_scope();
                for stmt in block {
                    self.check_statement(&stmt.inner)?;
                }
                self.exit_scope();
                if let Some(catch) = handler {
                    self.enter_scope();
                    self.define_variable(&catch.param, Type::Any);
                    for stmt in &catch.body {
                        self.check_statement(&stmt.inner)?;
                    }
                    self.exit_scope();
                }
                if let Some(fin) = finalizer {
                    for stmt in fin {
                        self.check_statement(&stmt.inner)?;
                    }
                }
                Ok(Type::Void)
            }
            Statement::ThrowStatement(expr) => {
                self.check_expression(expr)?;
                Ok(Type::Never)
            }
            Statement::ExportDeclaration { kind } => match kind {
                crate::compiler::parser::ExportDeclarationKind::Local(declaration) => {
                    self.check_statement(&declaration.inner)
                }
                crate::compiler::parser::ExportDeclarationKind::ReExport { .. } => Ok(Type::Void),
            },
            Statement::ExportDefaultDeclaration { declaration } => {
                self.check_statement(&declaration.inner)
            }
            Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::LabeledStatement { .. }
            | Statement::EmptyStatement => Ok(Type::Void),
            Statement::ImportDeclaration { .. } => Ok(Type::Void),
            Statement::DoWhileStatement { condition, body } => {
                self.check_expression(condition)?;
                self.check_statement(&body.inner)?;
                Ok(Type::Void)
            }
            &Statement::YieldStatement(_) => todo!(),
        }
    }
}
