use super::{ClassInfo, ClassMethodInfo, ClassMethodKind, Instruction};
use crate::compiler::parser::{ClassMember, Expression, SpannedNode, Statement};
use crate::errors::Result;

use super::CodeGenerator;

impl CodeGenerator {
    pub(super) fn generate_class_statement(&mut self, stmt: &Statement) -> Result<bool> {
        match stmt {
            Statement::ClassDeclaration {
                name,
                superclass,
                body,
            } => {
                let class_name = name.clone();

                let class_info_idx = self.class_infos.len() as u32;

                let constructor_func_idx = self.compile_class_constructor(body)?;

                let mut methods = Vec::new();
                for member in body {
                    match member {
                        ClassMember::Method {
                            name: mname,
                            params,
                            body: mbody,
                            is_static,
                            ..
                        } => {
                            let func_idx =
                                self.compile_function(Some(mname.clone()), params, mbody, false)?;
                            methods.push(ClassMethodInfo {
                                name: mname.clone(),
                                func_idx,
                                is_static: *is_static,
                                kind: ClassMethodKind::Method,
                            });
                        }
                        ClassMember::Getter {
                            name: mname,
                            body: mbody,
                            is_static,
                            ..
                        } => {
                            let func_idx = self.compile_function(
                                Some(format!("get_{}", mname)),
                                &[],
                                mbody,
                                false,
                            )?;
                            methods.push(ClassMethodInfo {
                                name: mname.clone(),
                                func_idx,
                                is_static: *is_static,
                                kind: ClassMethodKind::Getter,
                            });
                        }
                        ClassMember::Setter {
                            name: mname,
                            param,
                            body: mbody,
                            is_static,
                            ..
                        } => {
                            let func_idx = self.compile_function(
                                Some(format!("set_{}", mname)),
                                std::slice::from_ref(param),
                                mbody,
                                false,
                            )?;
                            methods.push(ClassMethodInfo {
                                name: mname.clone(),
                                func_idx,
                                is_static: *is_static,
                                kind: ClassMethodKind::Setter,
                            });
                        }
                        ClassMember::Constructor { .. } | ClassMember::Property { .. } => {}
                    }
                }

                let superclass_name = superclass.as_ref().and_then(|expr| {
                    if let Expression::Identifier(name) = expr.as_ref() {
                        Some(name.clone())
                    } else {
                        None
                    }
                });

                self.class_infos.push(ClassInfo {
                    name: class_name.clone(),
                    constructor_func_idx,
                    methods,
                    superclass: superclass_name,
                });

                if superclass.is_some() {
                    self.generate_expression(superclass.as_ref().unwrap())?;
                }

                self.emit(Instruction::MakeClass(class_info_idx));

                if self.scope_depth == 0 {
                    self.emit(Instruction::StoreGlobal(class_name));
                } else {
                    self.locals.push(class_name);
                    let slot = self.last_local_slot();
                    self.emit(Instruction::StoreLocal(slot));
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(super) fn compile_class_constructor(
        &mut self,
        body: &[ClassMember],
    ) -> Result<Option<u32>> {
        // Collect property initializers
        let mut prop_inits: Vec<(String, Option<Expression>)> = Vec::new();
        for member in body {
            if let ClassMember::Property { name, init, .. } = member {
                prop_inits.push((name.clone(), init.clone()));
            }
        }

        // Find explicit constructor
        for member in body {
            if let ClassMember::Constructor { params, body } = member {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                // If there are property initializers, we need to prepend them to the constructor body
                if prop_inits.is_empty() {
                    return Ok(Some(self.compile_function(
                        Some("constructor".to_string()),
                        &param_names,
                        body,
                        false,
                    )?));
                }
                // Build synthetic statements for property initializations
                let mut synthetic_body: Vec<SpannedNode<Statement>> = Vec::new();
                for (name, init_expr) in &prop_inits {
                    let value = if let Some(expr) = init_expr {
                        expr.clone()
                    } else {
                        Expression::UndefinedLiteral
                    };
                    synthetic_body.push(SpannedNode {
                        inner: Statement::Expression(Expression::Assignment {
                            target: Box::new(Expression::Member {
                                object: Box::new(Expression::Identifier("this".to_string())),
                                property: Box::new(Expression::Identifier(name.clone())),
                                computed: false,
                            }),
                            value: Box::new(value),
                            op: None,
                        }),
                        span: Some(crate::errors::Span::unknown()),
                    });
                }
                synthetic_body.extend(body.clone());
                return Ok(Some(self.compile_function(
                    Some("constructor".to_string()),
                    &param_names,
                    &synthetic_body,
                    false,
                )?));
            }
        }

        // No explicit constructor but has property initializers — create a default constructor
        if !prop_inits.is_empty() {
            let mut synthetic_body: Vec<SpannedNode<Statement>> = Vec::new();
            for (name, init_expr) in &prop_inits {
                let value = if let Some(expr) = init_expr {
                    expr.clone()
                } else {
                    Expression::UndefinedLiteral
                };
                synthetic_body.push(SpannedNode {
                    inner: Statement::Expression(Expression::Assignment {
                        target: Box::new(Expression::Member {
                            object: Box::new(Expression::Identifier("this".to_string())),
                            property: Box::new(Expression::Identifier(name.clone())),
                            computed: false,
                        }),
                        value: Box::new(value),
                        op: None,
                    }),
                    span: Some(crate::errors::Span::unknown()),
                });
            }
            return Ok(Some(self.compile_function(
                Some("constructor".to_string()),
                &[],
                &synthetic_body,
                false,
            )?));
        }

        Ok(None)
    }
}
