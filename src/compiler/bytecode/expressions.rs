use super::*;
use crate::errors::Result;

impl CodeGenerator {
    pub(crate) fn generate_expression(&mut self, expr: &Expression) -> Result<()> {
        match expr {
            Expression::NumberLiteral(n) => {
                let idx = self.add_constant(Value::Float(*n));
                self.instructions.push(Instruction::LoadConst(idx));
                Ok(())
            }
            Expression::StringLiteral(s) => {
                let idx = self.add_constant(Value::String(s.clone()));
                self.instructions.push(Instruction::LoadConst(idx));
                Ok(())
            }
            Expression::BooleanLiteral(b) => {
                if *b {
                    self.instructions.push(Instruction::LoadTrue);
                } else {
                    self.instructions.push(Instruction::LoadFalse);
                }
                Ok(())
            }
            Expression::NullLiteral => {
                self.instructions.push(Instruction::LoadNull);
                Ok(())
            }
            Expression::UndefinedLiteral => {
                self.instructions.push(Instruction::LoadUndefined);
                Ok(())
            }
            Expression::NaNLiteral => {
                let idx = self.add_constant(Value::Float(f64::NAN));
                self.instructions.push(Instruction::LoadConst(idx));
                Ok(())
            }
            Expression::Identifier(name) => {
                if name == "this" {
                    self.instructions.push(Instruction::LoadThis);
                } else if let Some(local_idx) = self.resolve_local(name) {
                    self.instructions.push(Instruction::LoadLocal(local_idx));
                } else {
                    self.instructions
                        .push(Instruction::LoadGlobal(name.clone()));
                }
                Ok(())
            }
            Expression::BinaryOp { op, left, right } => {
                self.generate_expression(left)?;
                self.generate_expression(right)?;
                self.generate_binary_op(op)?;
                Ok(())
            }
            Expression::UnaryOp { op, operand } => {
                match op {
                    UnaryOperator::Delete => {
                        if let Expression::Member {
                            object,
                            property,
                            computed,
                        } = operand.as_ref()
                        {
                            self.generate_expression(object)?;
                            if *computed {
                                self.generate_expression(property)?;
                            } else if let Expression::Identifier(name) = property.as_ref() {
                                let idx = self.add_constant(Value::String(name.clone()));
                                self.instructions.push(Instruction::LoadConst(idx));
                            } else {
                                self.generate_expression(property)?;
                            }
                            self.instructions.push(Instruction::Delete);
                        } else {
                            self.generate_expression(operand)?;
                            self.instructions.push(Instruction::Pop);
                            self.instructions.push(Instruction::LoadTrue);
                        }
                    }
                    UnaryOperator::Void
                        if matches!(operand.as_ref(), Expression::Assignment { .. }) =>
                    {
                        self.generate_expression(operand)?;
                        self.instructions.push(Instruction::Pop);
                        self.instructions.push(Instruction::LoadUndefined);
                    }
                    _ => {
                        // Special handling for typeof of identifier - doesn't throw for undeclared variables
                        if let UnaryOperator::Typeof = op {
                            if let Expression::Identifier(name) = operand.as_ref() {
                                if let Some(local_idx) = self.resolve_local(name) {
                                    self.instructions.push(Instruction::LoadLocal(local_idx));
                                } else {
                                    self.instructions
                                        .push(Instruction::TypeOfGlobal(name.clone()));
                                }
                                return Ok(());
                            }
                        }
                        self.generate_expression(operand)?;
                        match op {
                            UnaryOperator::Negate => self.instructions.push(Instruction::Negate),
                            UnaryOperator::Not => self.instructions.push(Instruction::Not),
                            UnaryOperator::Typeof => self.instructions.push(Instruction::TypeOf),
                            UnaryOperator::Void => self.instructions.push(Instruction::Void),
                            UnaryOperator::BitNot => self.instructions.push(Instruction::BitNot),
                            _ => {}
                        }
                    }
                }
                Ok(())
            }
            Expression::Assignment { target, value, op } => {
                if let Some(compound_op) = op {
                    self.generate_expression(target)?;
                    self.generate_expression(value)?;
                    match compound_op {
                        CompoundAssignmentOp::AddAssign => self.instructions.push(Instruction::Add),
                        CompoundAssignmentOp::SubAssign => self.instructions.push(Instruction::Sub),
                        CompoundAssignmentOp::MulAssign => self.instructions.push(Instruction::Mul),
                        CompoundAssignmentOp::DivAssign => self.instructions.push(Instruction::Div),
                        CompoundAssignmentOp::ModAssign => self.instructions.push(Instruction::Mod),
                        CompoundAssignmentOp::AndAssign => self.instructions.push(Instruction::And),
                        CompoundAssignmentOp::OrAssign => self.instructions.push(Instruction::Or),
                    }
                    if let Expression::Identifier(name) = target.as_ref() {
                        if let Some(local_idx) = self.resolve_local(name) {
                            self.instructions.push(Instruction::StoreLocal(local_idx));
                        } else {
                            self.instructions
                                .push(Instruction::StoreGlobal(name.clone()));
                        }
                    } else if let Expression::Member {
                        object,
                        property,
                        computed,
                    } = target.as_ref()
                    {
                        self.generate_expression(object)?;
                        if *computed {
                            self.generate_expression(property)?;
                        } else if let Expression::Identifier(name) = property.as_ref() {
                            let idx = self.add_constant(Value::String(name.clone()));
                            self.instructions.push(Instruction::LoadConst(idx));
                        } else {
                            self.generate_expression(property)?;
                        }
                        self.instructions.push(Instruction::SetProperty);
                        self.instructions.push(Instruction::Pop);
                    } else {
                        return Err(crate::errors::Error::RuntimeError(
                            "Invalid assignment target".into(),
                        ));
                    }
                } else {
                    if let Expression::Member {
                        object,
                        property,
                        computed,
                    } = target.as_ref()
                    {
                        self.generate_expression(object)?;
                        if *computed {
                            self.generate_expression(property)?;
                        } else if let Expression::Identifier(name) = property.as_ref() {
                            let idx = self.add_constant(Value::String(name.clone()));
                            self.instructions.push(Instruction::LoadConst(idx));
                        } else {
                            self.generate_expression(property)?;
                        }
                        self.generate_expression(value)?;
                        self.instructions.push(Instruction::SetProperty);
                    } else if let Expression::Identifier(name) = target.as_ref() {
                        self.generate_expression(value)?;
                        if let Some(local_idx) = self.resolve_local(name) {
                            self.instructions.push(Instruction::StoreLocal(local_idx));
                        } else {
                            self.instructions
                                .push(Instruction::StoreGlobal(name.clone()));
                        }
                    } else {
                        return Err(crate::errors::Error::RuntimeError(
                            "Invalid assignment target".into(),
                        ));
                    }
                }
                Ok(())
            }
            Expression::Call { callee, args } => {
                if let Expression::Member {
                    object,
                    property,
                    computed,
                } = callee.as_ref()
                {
                    self.generate_expression(object)?;
                    if *computed {
                        self.generate_expression(property)?;
                    } else if let Expression::Identifier(name) = property.as_ref() {
                        let idx = self.add_constant(Value::String(name.clone()));
                        self.instructions.push(Instruction::LoadConst(idx));
                    } else {
                        self.generate_expression(property)?;
                    }
                    for arg in args {
                        self.generate_expression(arg)?;
                    }
                    self.instructions
                        .push(Instruction::CallMethod(args.len() as u16));
                } else {
                    for arg in args {
                        self.generate_expression(arg)?;
                    }
                    self.generate_expression(callee)?;
                    self.instructions.push(Instruction::Call(args.len() as u16));
                }
                Ok(())
            }
            Expression::Member {
                object,
                property,
                computed,
            } => {
                self.generate_expression(object)?;
                if *computed {
                    self.generate_expression(property)?;
                } else if let Expression::Identifier(name) = property.as_ref() {
                    let idx = self.add_constant(Value::String(name.clone()));
                    self.instructions.push(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.instructions.push(Instruction::GetProperty);
                Ok(())
            }
            Expression::OptionalMember {
                object,
                property,
                computed,
            } => {
                self.generate_expression(object)?;
                self.instructions.push(Instruction::Dup);
                let check_undef = self.instructions.len();
                self.instructions.push(Instruction::JumpIfUndefined(0));
                if *computed {
                    self.generate_expression(property)?;
                } else if let Expression::Identifier(name) = property.as_ref() {
                    let idx = self.add_constant(Value::String(name.clone()));
                    self.instructions.push(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.instructions.push(Instruction::GetProperty);
                let skip_end = self.instructions.len();
                self.instructions.push(Instruction::Jump(0));
                self.patch_jump(check_undef, self.instructions.len());
                self.instructions.push(Instruction::Pop);
                self.instructions.push(Instruction::LoadUndefined);
                self.patch_jump(skip_end, self.instructions.len());
                Ok(())
            }
            Expression::OptionalCall { callee, args } => {
                if let Expression::Member {
                    object,
                    property,
                    computed,
                } = callee.as_ref()
                {
                    self.generate_expression(object)?;
                    self.instructions.push(Instruction::Dup);
                    let check_undef = self.instructions.len();
                    self.instructions.push(Instruction::JumpIfUndefined(0));
                    if *computed {
                        self.generate_expression(property)?;
                    } else if let Expression::Identifier(name) = property.as_ref() {
                        let idx = self.add_constant(Value::String(name.clone()));
                        self.instructions.push(Instruction::LoadConst(idx));
                    } else {
                        self.generate_expression(property)?;
                    }
                    for arg in args {
                        self.generate_expression(arg)?;
                    }
                    self.instructions
                        .push(Instruction::CallMethod(args.len() as u16));
                    let skip_end = self.instructions.len();
                    self.instructions.push(Instruction::Jump(0));
                    self.patch_jump(check_undef, self.instructions.len());
                    self.instructions.push(Instruction::Pop);
                    self.instructions.push(Instruction::LoadUndefined);
                    self.patch_jump(skip_end, self.instructions.len());
                } else {
                    self.generate_expression(callee)?;
                    self.instructions.push(Instruction::Dup);
                    let check_undef = self.instructions.len();
                    self.instructions.push(Instruction::JumpIfUndefined(0));
                    for arg in args {
                        self.generate_expression(arg)?;
                    }
                    self.instructions.push(Instruction::Call(args.len() as u16));
                    let skip_end = self.instructions.len();
                    self.instructions.push(Instruction::Jump(0));
                    self.patch_jump(check_undef, self.instructions.len());
                    self.instructions.push(Instruction::Pop);
                    self.instructions.push(Instruction::LoadUndefined);
                    self.patch_jump(skip_end, self.instructions.len());
                }
                Ok(())
            }
            Expression::FunctionExpression {
                name: _,
                params,
                body,
                is_async: _,
                param_types: _,
                return_type: _,
                is_generator,
            } => {
                let func_idx = self.functions.len() as u32;
                let parent_locals_snapshot = self.locals.clone();
                let outer_refs =
                    super::closures::find_outer_refs(body, params, &parent_locals_snapshot);
                let num_captures = outer_refs.len();

                self.functions.push(CompiledFunction {
                    name: None,
                    params: params.clone(),
                    bytecode_index: 0,
                    param_count: params.len(),
                    closure_var_count: num_captures,
                    is_generator: *is_generator,
                });

                let jump_over = self.instructions.len();
                self.instructions.push(Instruction::Jump(0));

                let func_start = self.instructions.len();
                self.functions[func_idx as usize].bytecode_index = func_start;

                self.scope_depth += 1;
                let prev_locals = self.locals.len();

                let saved_captured = std::mem::take(&mut self.captured_var_names);
                let saved_start = self.local_start_idx;
                self.captured_var_names = outer_refs.iter().map(|(n, _)| n.clone()).collect();
                self.local_start_idx = self.locals.len();

                for param in params {
                    self.locals.push(param.clone());
                }

                for stmt in body {
                    self.generate_statement(stmt, false)?;
                }

                self.instructions.push(Instruction::LoadUndefined);
                self.instructions.push(Instruction::Return);

                self.scope_depth -= 1;
                self.locals.truncate(prev_locals);
                self.captured_var_names = saved_captured;
                self.local_start_idx = saved_start;

                self.patch_jump(jump_over, self.instructions.len());

                if num_captures > 0 {
                    let capture_slots: Vec<u16> = outer_refs.iter().map(|(_, s)| *s).collect();
                    self.instructions
                        .push(Instruction::MakeClosure(func_idx, capture_slots));
                } else {
                    self.instructions.push(Instruction::MakeFunction(func_idx));
                }
                Ok(())
            }
            Expression::ArrowFunction {
                params,
                body,
                is_async: _,
                param_types: _,
                return_type: _,
            } => {
                let func_idx = self.functions.len() as u32;

                let (body_stmts, is_expr) = match body.as_ref() {
                    ArrowFunctionBody::Expression(expr) => {
                        (vec![Statement::ReturnStatement(Some(expr.clone()))], true)
                    }
                    ArrowFunctionBody::Block(stmts) => (stmts.clone(), false),
                };

                let parent_locals_snapshot = self.locals.clone();
                let outer_refs =
                    super::closures::find_outer_refs(&body_stmts, params, &parent_locals_snapshot);
                let num_captures = outer_refs.len();

                self.functions.push(CompiledFunction {
                    name: None,
                    params: params.clone(),
                    bytecode_index: 0,
                    param_count: params.len(),
                    closure_var_count: num_captures,
                    is_generator: false,
                });

                let jump_over = self.instructions.len();
                self.instructions.push(Instruction::Jump(0));

                let func_start = self.instructions.len();
                self.functions[func_idx as usize].bytecode_index = func_start;

                self.scope_depth += 1;
                let prev_locals = self.locals.len();

                let saved_captured = std::mem::take(&mut self.captured_var_names);
                let saved_start = self.local_start_idx;
                self.captured_var_names = outer_refs.iter().map(|(n, _)| n.clone()).collect();
                self.local_start_idx = self.locals.len();

                for param in params {
                    self.locals.push(param.clone());
                }

                for stmt in &body_stmts {
                    self.generate_statement(stmt, false)?;
                }

                if is_expr {
                    // already return statements
                } else {
                    self.instructions.push(Instruction::LoadUndefined);
                    self.instructions.push(Instruction::Return);
                }

                self.scope_depth -= 1;
                self.locals.truncate(prev_locals);
                self.captured_var_names = saved_captured;
                self.local_start_idx = saved_start;

                self.patch_jump(jump_over, self.instructions.len());

                if num_captures > 0 {
                    let capture_slots: Vec<u16> = outer_refs.iter().map(|(_, s)| *s).collect();
                    self.instructions
                        .push(Instruction::MakeClosure(func_idx, capture_slots));
                } else {
                    self.instructions.push(Instruction::MakeFunction(func_idx));
                }
                Ok(())
            }
            Expression::NewExpression { callee, args } => {
                self.generate_expression(callee)?;
                for arg in args {
                    self.generate_expression(arg)?;
                }
                self.instructions
                    .push(Instruction::Construct(args.len() as u16));
                Ok(())
            }
            Expression::ConditionalExpression {
                test,
                consequent,
                alternate,
            } => {
                self.generate_expression(test)?;
                let jump_if_not = self.instructions.len();
                self.instructions.push(Instruction::JumpIfNot(0));
                self.generate_expression(consequent)?;
                let jump_to_end = self.instructions.len();
                self.instructions.push(Instruction::Jump(0));
                self.patch_jump(jump_if_not, self.instructions.len());
                self.generate_expression(alternate)?;
                self.patch_jump(jump_to_end, self.instructions.len());
                Ok(())
            }
            Expression::UpdateExpression {
                op,
                operand,
                prefix,
            } => {
                if let Expression::Identifier(name) = operand.as_ref() {
                    if *prefix {
                        self.generate_expression(operand)?;
                        let one = self.add_constant(Value::Float(1.0));
                        self.instructions.push(Instruction::LoadConst(one));
                        match op {
                            UpdateOperator::Increment => self.instructions.push(Instruction::Add),
                            UpdateOperator::Decrement => self.instructions.push(Instruction::Sub),
                        }
                        if let Some(local_idx) = self.resolve_local(name) {
                            self.instructions.push(Instruction::StoreLocal(local_idx));
                        } else {
                            self.instructions
                                .push(Instruction::StoreGlobal(name.clone()));
                        }
                    } else {
                        self.generate_expression(operand)?;
                        if let Some(local_idx) = self.resolve_local(name) {
                            self.instructions.push(Instruction::LoadLocal(local_idx));
                        } else {
                            self.instructions
                                .push(Instruction::LoadGlobal(name.clone()));
                        }
                        let one = self.add_constant(Value::Float(1.0));
                        self.instructions.push(Instruction::LoadConst(one));
                        match op {
                            UpdateOperator::Increment => self.instructions.push(Instruction::Add),
                            UpdateOperator::Decrement => self.instructions.push(Instruction::Sub),
                        }
                        if let Some(local_idx) = self.resolve_local(name) {
                            self.instructions.push(Instruction::StoreLocal(local_idx));
                        } else {
                            self.instructions
                                .push(Instruction::StoreGlobal(name.clone()));
                        }
                    }
                } else {
                    self.generate_expression(operand)?;
                }
                Ok(())
            }
            Expression::TemplateLiteral {
                quasis,
                expressions,
            } => {
                if expressions.is_empty() {
                    let s = quasis.join("");
                    let idx = self.add_constant(Value::String(s));
                    self.instructions.push(Instruction::LoadConst(idx));
                } else {
                    let first = &quasis[0];
                    if !first.is_empty() {
                        let idx = self.add_constant(Value::String(first.clone()));
                        self.instructions.push(Instruction::LoadConst(idx));
                    }

                    for i in 0..expressions.len() {
                        if first.is_empty() && i == 0 {
                            self.generate_expression(&expressions[i])?;
                            self.instructions.push(Instruction::ToString);
                        } else {
                            self.generate_expression(&expressions[i])?;
                            self.instructions.push(Instruction::ToString);
                            self.instructions.push(Instruction::Add);
                        }

                        if i + 1 < quasis.len() && !quasis[i + 1].is_empty() {
                            let idx = self.add_constant(Value::String(quasis[i + 1].clone()));
                            self.instructions.push(Instruction::LoadConst(idx));
                            self.instructions.push(Instruction::Add);
                        }
                    }
                }
                Ok(())
            }
            Expression::ClassExpression {
                name,
                superclass,
                body,
            } => {
                let class_info_idx = self.class_infos.len() as u32;
                let class_name = name.clone().unwrap_or_else(|| "anonymous".to_string());

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
                    name: class_name,
                    constructor_func_idx,
                    methods,
                    superclass: superclass_name,
                });

                if superclass.is_some() {
                    self.generate_expression(superclass.as_ref().unwrap())?;
                }

                self.instructions
                    .push(Instruction::MakeClass(class_info_idx));
                Ok(())
            }
            Expression::AwaitExpression { argument } => {
                self.generate_expression(argument)?;
                self.instructions.push(Instruction::Await);
                Ok(())
            }
            Expression::SuperCall { args } => {
                self.instructions.push(Instruction::LoadThis);
                for arg in args {
                    self.generate_expression(arg)?;
                }
                self.instructions
                    .push(Instruction::SuperConstruct(args.len() as u16));
                Ok(())
            }
            Expression::SuperMember { property, computed } => {
                self.instructions.push(Instruction::LoadThis);
                if *computed {
                    self.generate_expression(property)?;
                } else if let Expression::Identifier(name) = property.as_ref() {
                    let idx = self.add_constant(Value::String(name.clone()));
                    self.instructions.push(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.instructions.push(Instruction::SuperGet);
                Ok(())
            }
            Expression::ArrayLiteral { elements } => {
                let has_spread = elements
                    .iter()
                    .any(|e| matches!(e, Expression::SpreadElement { .. }));
                if has_spread {
                    self.instructions.push(Instruction::NewArray(0));
                    for elem in elements {
                        match elem {
                            Expression::SpreadElement { argument } => {
                                self.instructions.push(Instruction::Dup);
                                self.generate_expression(argument)?;
                                self.instructions.push(Instruction::SpreadArray);
                            }
                            _ => {
                                self.instructions.push(Instruction::Dup);
                                self.generate_expression(elem)?;
                                self.instructions.push(Instruction::ArrayPush);
                            }
                        }
                    }
                } else {
                    for elem in elements.iter().rev() {
                        self.generate_expression(elem)?;
                    }
                    self.instructions
                        .push(Instruction::NewArray(elements.len() as u32));
                }
                Ok(())
            }
            Expression::ObjectLiteral { properties } => {
                let has_spread = properties.iter().any(|p| p.key.is_empty());
                if has_spread {
                    self.instructions.push(Instruction::NewObject);
                    for prop in properties {
                        if prop.key.is_empty()
                            && matches!(prop.value, Expression::SpreadElement { .. })
                        {
                            if let Expression::SpreadElement { argument } = &prop.value {
                                self.instructions.push(Instruction::Dup);
                                self.generate_expression(argument)?;
                                self.instructions.push(Instruction::SpreadObject);
                            }
                        } else {
                            if prop.computed {
                                if let Some(key_expr) = &prop.computed_key {
                                    self.generate_expression(key_expr)?;
                                }
                            } else {
                                let key_idx = self.add_constant(Value::String(prop.key.clone()));
                                self.instructions.push(Instruction::LoadConst(key_idx));
                            }
                            self.generate_expression(&prop.value)?;
                            self.instructions.push(Instruction::SetProperty);
                        }
                    }
                } else {
                    self.instructions.push(Instruction::NewObject);
                    for prop in properties {
                        if prop.computed {
                            if let Some(key_expr) = &prop.computed_key {
                                self.generate_expression(key_expr)?;
                                self.instructions.push(Instruction::ToString);
                            }
                        } else {
                            let key_idx = self.add_constant(Value::String(prop.key.clone()));
                            self.instructions.push(Instruction::LoadConst(key_idx));
                        }
                        self.generate_expression(&prop.value)?;
                        self.instructions.push(Instruction::SetProperty);
                    }
                }
                Ok(())
            }
            Expression::SpreadElement { argument } => {
                self.generate_expression(argument)?;
                Ok(())
            }
            Expression::RestElement { .. } => Ok(()),
            Expression::TypeAssertion {
                expression,
                type_annotation: _,
            } => {
                self.generate_expression(expression)?;
                Ok(())
            }
        }
    }

    fn generate_binary_op(&mut self, op: &BinaryOperator) -> Result<()> {
        match op {
            BinaryOperator::Add => self.instructions.push(Instruction::Add),
            BinaryOperator::Sub => self.instructions.push(Instruction::Sub),
            BinaryOperator::Mul => self.instructions.push(Instruction::Mul),
            BinaryOperator::Div => self.instructions.push(Instruction::Div),
            BinaryOperator::Mod => self.instructions.push(Instruction::Mod),
            BinaryOperator::Power => self.instructions.push(Instruction::Power),
            BinaryOperator::Eq => self.instructions.push(Instruction::Eq),
            BinaryOperator::StrictEq => self.instructions.push(Instruction::StrictEq),
            BinaryOperator::NotEqual => self.instructions.push(Instruction::NotEqual),
            BinaryOperator::StrictNotEqual => self.instructions.push(Instruction::StrictNotEqual),
            BinaryOperator::Less => self.instructions.push(Instruction::Less),
            BinaryOperator::Greater => self.instructions.push(Instruction::Greater),
            BinaryOperator::LessEqual => self.instructions.push(Instruction::LessEqual),
            BinaryOperator::GreaterEqual => self.instructions.push(Instruction::GreaterEqual),
            BinaryOperator::And => self.instructions.push(Instruction::And),
            BinaryOperator::Or => self.instructions.push(Instruction::Or),
            BinaryOperator::Instanceof => self.instructions.push(Instruction::InstanceOf),
            BinaryOperator::In => self.instructions.push(Instruction::In),
            BinaryOperator::NullishCoalescing => {
                self.instructions.push(Instruction::NullishCoalescing)
            }
            _ => {}
        }
        Ok(())
    }
}
