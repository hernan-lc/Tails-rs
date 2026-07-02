use super::*;
use crate::errors::Result;

impl CodeGenerator {
    pub(crate) fn generate_expression(&mut self, expr: &Expression) -> Result<()> {
        match expr {
            Expression::NumberLiteral(n) => {
                let idx = self.add_constant(Value::Float(*n));
                self.emit(Instruction::LoadConst(idx));
                Ok(())
            }
            Expression::BigIntLiteral(s) => {
                let val: i128 = s.parse().unwrap_or(0);
                let idx = self.add_constant(Value::BigInt(val));
                self.emit(Instruction::LoadConst(idx));
                Ok(())
            }
            Expression::StringLiteral(s) => {
                let idx = self.add_constant(Value::String(s.clone()));
                self.emit(Instruction::LoadConst(idx));
                Ok(())
            }
            Expression::RegexLiteral { pattern, flags } => {
                // Create a RegExp object: new RegExp(pattern, flags)
                // Push constructor first (below args on stack)
                self.emit(Instruction::LoadGlobal("RegExp".to_string()));
                let reg_idx = self.add_constant(Value::String(pattern.clone()));
                self.emit(Instruction::LoadConst(reg_idx));
                if !flags.is_empty() {
                    let flags_idx = self.add_constant(Value::String(flags.clone()));
                    self.emit(Instruction::LoadConst(flags_idx));
                    self.emit(Instruction::Construct(2));
                } else {
                    self.emit(Instruction::Construct(1));
                }
                Ok(())
            }
            Expression::BooleanLiteral(b) => {
                if *b {
                    self.emit(Instruction::LoadTrue);
                } else {
                    self.emit(Instruction::LoadFalse);
                }
                Ok(())
            }
            Expression::NullLiteral => {
                self.emit(Instruction::LoadNull);
                Ok(())
            }
            Expression::UndefinedLiteral => {
                self.emit(Instruction::LoadUndefined);
                Ok(())
            }
            Expression::NaNLiteral => {
                let idx = self.add_constant(Value::Float(f64::NAN));
                self.emit(Instruction::LoadConst(idx));
                Ok(())
            }
            Expression::InfinityLiteral => {
                let idx = self.add_constant(Value::Float(f64::INFINITY));
                self.emit(Instruction::LoadConst(idx));
                Ok(())
            }
            Expression::Identifier(name) => {
                if name == "this" {
                    self.emit(Instruction::LoadThis);
                } else if let Some(local_idx) = self.resolve_local(name) {
                    self.emit(Instruction::LoadLocal(local_idx));
                } else {
                    self.emit(Instruction::LoadGlobal(name.clone()));
                }
                Ok(())
            }
            Expression::BinaryOp { op, left, right } => {
                match op {
                    BinaryOperator::And => {
                        // Short-circuit &&: if left is falsy, return left without evaluating right
                        self.generate_expression(left)?;
                        self.emit(Instruction::Dup);
                        let skip_right = self.instructions.len();
                        self.emit(Instruction::JumpIfNot(0));
                        self.emit(Instruction::Pop);
                        self.generate_expression(right)?;
                        let done = self.instructions.len();
                        self.emit(Instruction::Jump(0));
                        self.patch_jump(skip_right, self.instructions.len());
                        self.patch_jump(done, self.instructions.len());
                    }
                    BinaryOperator::Or => {
                        // Short-circuit ||: if left is truthy, return left without evaluating right
                        self.generate_expression(left)?;
                        self.emit(Instruction::Dup);
                        let skip_right = self.instructions.len();
                        self.emit(Instruction::JumpIf(0));
                        self.emit(Instruction::Pop);
                        self.generate_expression(right)?;
                        let done = self.instructions.len();
                        self.emit(Instruction::Jump(0));
                        self.patch_jump(skip_right, self.instructions.len());
                        self.patch_jump(done, self.instructions.len());
                    }
                    BinaryOperator::NullishCoalescing => {
                        // Short-circuit ??: if left is not null/undefined, return left without evaluating right
                        // JumpIfNotUndefined peeks (doesn't pop), so no dup needed
                        self.generate_expression(left)?;
                        let skip_right = self.instructions.len();
                        self.emit(Instruction::JumpIfNotUndefined(0));
                        self.emit(Instruction::Pop);
                        self.generate_expression(right)?;
                        let done = self.instructions.len();
                        self.emit(Instruction::Jump(0));
                        self.patch_jump(skip_right, self.instructions.len());
                        self.patch_jump(done, self.instructions.len());
                    }
                    _ => {
                        self.generate_expression(left)?;
                        self.generate_expression(right)?;
                        self.generate_binary_op(op)?;
                    }
                }
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
                                self.emit(Instruction::LoadConst(idx));
                            } else {
                                self.generate_expression(property)?;
                            }
                            self.emit(Instruction::Delete);
                        } else {
                            self.generate_expression(operand)?;
                            self.emit(Instruction::Pop);
                            self.emit(Instruction::LoadTrue);
                        }
                    }
                    UnaryOperator::Void
                        if matches!(operand.as_ref(), Expression::Assignment { .. }) =>
                    {
                        self.generate_expression(operand)?;
                        self.emit(Instruction::Pop);
                        self.emit(Instruction::LoadUndefined);
                    }
                    _ => {
                        // Special handling for typeof of identifier - doesn't throw for undeclared variables
                        if let UnaryOperator::Typeof = op {
                            if let Expression::Identifier(name) = operand.as_ref() {
                                if let Some(local_idx) = self.resolve_local(name) {
                                    self.emit(Instruction::LoadLocal(local_idx));
                                    self.emit(Instruction::TypeOf);
                                } else {
                                    self.emit(Instruction::TypeOfGlobal(name.clone()));
                                }
                                return Ok(());
                            }
                        }
                        self.generate_expression(operand)?;
                        match op {
                            UnaryOperator::Negate => self.emit(Instruction::Negate),
                            UnaryOperator::Not => self.emit(Instruction::Not),
                            UnaryOperator::Typeof => self.emit(Instruction::TypeOf),
                            UnaryOperator::Void => self.emit(Instruction::Void),
                            UnaryOperator::BitNot => self.emit(Instruction::BitNot),
                            UnaryOperator::Delete => {}
                            UnaryOperator::UnaryPlus => {}
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
                        CompoundAssignmentOp::AddAssign => self.emit(Instruction::Add),
                        CompoundAssignmentOp::SubAssign => self.emit(Instruction::Sub),
                        CompoundAssignmentOp::MulAssign => self.emit(Instruction::Mul),
                        CompoundAssignmentOp::DivAssign => self.emit(Instruction::Div),
                        CompoundAssignmentOp::ModAssign => self.emit(Instruction::Mod),
                        CompoundAssignmentOp::AndAssign => self.emit(Instruction::And),
                        CompoundAssignmentOp::OrAssign => self.emit(Instruction::Or),
                        CompoundAssignmentOp::XorAssign => self.emit(Instruction::BitXor),
                        CompoundAssignmentOp::BitAndAssign => self.emit(Instruction::BitAnd),
                        CompoundAssignmentOp::BitOrAssign => self.emit(Instruction::BitOr),
                        CompoundAssignmentOp::NullishCoalescingAssign => {
                            self.emit(Instruction::NullishCoalescing)
                        }
                    }
                    if let Expression::Identifier(name) = target.as_ref() {
                        self.emit(Instruction::Dup);
                        if let Some(local_idx) = self.resolve_local(name) {
                            self.emit(Instruction::StoreLocal(local_idx));
                        } else {
                            self.emit(Instruction::StoreGlobal(name.clone()));
                        }
                    } else if let Expression::Member {
                        object,
                        property,
                        computed,
                    } = target.as_ref()
                    {
                        self.emit(Instruction::Dup);
                        self.generate_expression(object)?;
                        if *computed {
                            self.generate_expression(property)?;
                        } else if let Expression::Identifier(name) = property.as_ref() {
                            let idx = self.add_constant(Value::String(name.clone()));
                            self.emit(Instruction::LoadConst(idx));
                        } else {
                            self.generate_expression(property)?;
                        }
                        self.emit(Instruction::Rot3Right);
                        self.emit(Instruction::SetProperty);
                        self.emit(Instruction::Pop);
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
                            self.emit(Instruction::LoadConst(idx));
                        } else {
                            self.generate_expression(property)?;
                        }
                        self.generate_expression(value)?;
                        self.emit(Instruction::SetProperty);
                    } else if let Expression::Identifier(name) = target.as_ref() {
                        self.generate_expression(value)?;
                        self.emit(Instruction::Dup);
                        if let Some(local_idx) = self.resolve_local(name) {
                            self.emit(Instruction::StoreLocal(local_idx));
                        } else {
                            self.emit(Instruction::StoreGlobal(name.clone()));
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
                        self.emit(Instruction::LoadConst(idx));
                    } else {
                        self.generate_expression(property)?;
                    }
                    for arg in args {
                        self.generate_expression(arg)?;
                    }
                    self.emit(Instruction::CallMethod(args.len() as u16));
                } else if let Expression::OptionalMember {
                    object,
                    property,
                    computed,
                } = callee.as_ref()
                {
                    self.generate_expression(object)?;
                    self.emit(Instruction::Dup);
                    let check_undef = self.instructions.len();
                    self.emit(Instruction::JumpIfUndefined(0));
                    if *computed {
                        self.generate_expression(property)?;
                    } else if let Expression::Identifier(name) = property.as_ref() {
                        let idx = self.add_constant(Value::String(name.clone()));
                        self.emit(Instruction::LoadConst(idx));
                    } else {
                        self.generate_expression(property)?;
                    }
                    for arg in args {
                        self.generate_expression(arg)?;
                    }
                    self.emit(Instruction::CallMethod(args.len() as u16));
                    let skip_end = self.instructions.len();
                    self.emit(Instruction::Jump(0));
                    self.patch_jump(check_undef, self.instructions.len());
                    self.emit(Instruction::Pop);
                    self.emit(Instruction::LoadUndefined);
                    self.patch_jump(skip_end, self.instructions.len());
                } else {
                    for arg in args {
                        self.generate_expression(arg)?;
                    }
                    self.generate_expression(callee)?;
                    self.emit(Instruction::Call(args.len() as u16));
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
                    self.emit(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.emit(Instruction::GetProperty);
                Ok(())
            }
            Expression::OptionalMember {
                object,
                property,
                computed,
            } => {
                self.generate_expression(object)?;
                self.emit(Instruction::Dup);
                let check_undef = self.instructions.len();
                self.emit(Instruction::JumpIfUndefined(0));
                if *computed {
                    self.generate_expression(property)?;
                } else if let Expression::Identifier(name) = property.as_ref() {
                    let idx = self.add_constant(Value::String(name.clone()));
                    self.emit(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.emit(Instruction::GetProperty);
                let skip_end = self.instructions.len();
                self.emit(Instruction::Jump(0));
                self.patch_jump(check_undef, self.instructions.len());
                self.emit(Instruction::Pop);
                self.emit(Instruction::LoadUndefined);
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
                    self.emit(Instruction::Dup);
                    let check_undef = self.instructions.len();
                    self.emit(Instruction::JumpIfUndefined(0));
                    if *computed {
                        self.generate_expression(property)?;
                    } else if let Expression::Identifier(name) = property.as_ref() {
                        let idx = self.add_constant(Value::String(name.clone()));
                        self.emit(Instruction::LoadConst(idx));
                    } else {
                        self.generate_expression(property)?;
                    }
                    for arg in args {
                        self.generate_expression(arg)?;
                    }
                    self.emit(Instruction::CallMethod(args.len() as u16));
                    let skip_end = self.instructions.len();
                    self.emit(Instruction::Jump(0));
                    self.patch_jump(check_undef, self.instructions.len());
                    self.emit(Instruction::Pop);
                    self.emit(Instruction::LoadUndefined);
                    self.patch_jump(skip_end, self.instructions.len());
                } else if let Expression::OptionalMember {
                    object,
                    property,
                    computed,
                } = callee.as_ref()
                {
                    self.generate_expression(object)?;
                    self.emit(Instruction::Dup);
                    let check_undef = self.instructions.len();
                    self.emit(Instruction::JumpIfUndefined(0));
                    if *computed {
                        self.generate_expression(property)?;
                    } else if let Expression::Identifier(name) = property.as_ref() {
                        let idx = self.add_constant(Value::String(name.clone()));
                        self.emit(Instruction::LoadConst(idx));
                    } else {
                        self.generate_expression(property)?;
                    }
                    for arg in args {
                        self.generate_expression(arg)?;
                    }
                    self.emit(Instruction::CallMethod(args.len() as u16));
                    let skip_end = self.instructions.len();
                    self.emit(Instruction::Jump(0));
                    self.patch_jump(check_undef, self.instructions.len());
                    self.emit(Instruction::Pop);
                    self.emit(Instruction::LoadUndefined);
                    self.patch_jump(skip_end, self.instructions.len());
                } else {
                    self.generate_expression(callee)?;
                    self.emit(Instruction::Dup);
                    let check_undef = self.instructions.len();
                    self.emit(Instruction::JumpIfUndefined(0));
                    for arg in args {
                        self.generate_expression(arg)?;
                    }
                    self.emit(Instruction::Call(args.len() as u16));
                    let skip_end = self.instructions.len();
                    self.emit(Instruction::Jump(0));
                    self.patch_jump(check_undef, self.instructions.len());
                    self.emit(Instruction::Pop);
                    self.emit(Instruction::LoadUndefined);
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
                defaults: _,
                rest_param,
            } => {
                let func_idx = self.functions.len() as u32;
                let parent_locals_snapshot = self.locals.clone();
                let mut all_params = params.clone();
                if let Some(rp) = rest_param {
                    all_params.push(rp.clone());
                }
                let outer_refs =
                    super::closures::find_outer_refs(body, &all_params, &parent_locals_snapshot);
                let num_captures = outer_refs.len();

                self.functions.push(CompiledFunction {
                    name: None,
                    params: params.clone(),
                    rest_param: rest_param.clone(),
                    bytecode_index: 0,
                    param_count: params.len(),
                    closure_var_count: num_captures,
                    is_generator: *is_generator,
                    source_line: self.current_source_line,
                    is_arrow: false,
                });

                let jump_over = self.instructions.len();
                self.emit(Instruction::Jump(0));

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
                if let Some(rp) = rest_param {
                    self.locals.push(rp.clone());
                }

                for stmt in body {
                    self.record_line_from_span(&stmt.span);
                    self.generate_statement(&stmt.inner, false)?;
                }

                self.emit(Instruction::LoadUndefined);
                self.emit(Instruction::Return);

                self.scope_depth -= 1;
                self.locals.truncate(prev_locals);
                self.captured_var_names = saved_captured;
                self.local_start_idx = saved_start;

                self.patch_jump(jump_over, self.instructions.len());

                if num_captures > 0 {
                    let capture_slots: Vec<u16> = outer_refs.iter().map(|(_, s)| *s).collect();
                    self.emit(Instruction::MakeClosure(func_idx, capture_slots));
                } else {
                    self.emit(Instruction::MakeFunction(func_idx));
                }
                Ok(())
            }
            Expression::ArrowFunction {
                params,
                body,
                is_async: _,
                param_types: _,
                return_type: _,
                defaults: _,
                rest_param,
            } => {
                let func_idx = self.functions.len() as u32;

                let (body_stmts, is_expr) = match body.as_ref() {
                    ArrowFunctionBody::Expression(expr) => (
                        vec![SpannedNode {
                            inner: Statement::ReturnStatement(Some(expr.clone())),
                            span: None,
                        }],
                        true,
                    ),
                    ArrowFunctionBody::Block(stmts) => (stmts.clone(), false),
                };

                let parent_locals_snapshot = self.locals.clone();
                let mut all_params = params.clone();
                if let Some(rp) = rest_param {
                    all_params.push(rp.clone());
                }
                let outer_refs = super::closures::find_outer_refs(
                    &body_stmts,
                    &all_params,
                    &parent_locals_snapshot,
                );
                let num_captures = outer_refs.len();

                self.functions.push(CompiledFunction {
                    name: None,
                    params: params.clone(),
                    rest_param: rest_param.clone(),
                    bytecode_index: 0,
                    param_count: params.len(),
                    closure_var_count: num_captures,
                    is_generator: false,
                    source_line: self.current_source_line,
                    is_arrow: true,
                });

                let jump_over = self.instructions.len();
                self.emit(Instruction::Jump(0));

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
                if let Some(rp) = rest_param {
                    self.locals.push(rp.clone());
                }

                for stmt in &body_stmts {
                    self.record_line_from_span(&stmt.span);
                    self.generate_statement(&stmt.inner, false)?;
                }

                if is_expr {
                    // already return statements
                } else {
                    self.emit(Instruction::LoadUndefined);
                    self.emit(Instruction::Return);
                }

                self.scope_depth -= 1;
                self.locals.truncate(prev_locals);
                self.captured_var_names = saved_captured;
                self.local_start_idx = saved_start;

                self.patch_jump(jump_over, self.instructions.len());

                if num_captures > 0 {
                    let capture_slots: Vec<u16> = outer_refs.iter().map(|(_, s)| *s).collect();
                    self.emit(Instruction::MakeClosure(func_idx, capture_slots));
                } else {
                    self.emit(Instruction::MakeFunction(func_idx));
                }
                Ok(())
            }
            Expression::NewExpression { callee, args } => {
                self.generate_expression(callee)?;
                for arg in args {
                    self.generate_expression(arg)?;
                }
                self.emit(Instruction::Construct(args.len() as u16));
                Ok(())
            }
            Expression::ConditionalExpression {
                test,
                consequent,
                alternate,
            } => {
                self.generate_expression(test)?;
                let jump_if_not = self.instructions.len();
                self.emit(Instruction::JumpIfNot(0));
                self.generate_expression(consequent)?;
                let jump_to_end = self.instructions.len();
                self.emit(Instruction::Jump(0));
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
                    // Optimize: ++i / i++ for local variables -> IncLocal (fuses 4 instructions into 2)
                    if let Some(local_idx) = self.resolve_local(name) {
                        let delta = match op {
                            UpdateOperator::Increment => 1i64,
                            UpdateOperator::Decrement => -1i64,
                        };
                        if *prefix {
                            self.emit(Instruction::IncLocal(local_idx, delta));
                            self.emit(Instruction::LoadLocal(local_idx));
                        } else {
                            self.emit(Instruction::LoadLocal(local_idx));
                            self.emit(Instruction::IncLocal(local_idx, delta));
                        }
                    } else if *prefix {
                        self.generate_expression(operand)?;
                        let one = self.add_constant(Value::Float(1.0));
                        self.emit(Instruction::LoadConst(one));
                        match op {
                            UpdateOperator::Increment => self.emit(Instruction::Add),
                            UpdateOperator::Decrement => self.emit(Instruction::Sub),
                        }
                        self.emit(Instruction::StoreGlobal(name.clone()));
                    } else {
                        self.generate_expression(operand)?;
                        self.emit(Instruction::LoadGlobal(name.clone()));
                        let one = self.add_constant(Value::Float(1.0));
                        self.emit(Instruction::LoadConst(one));
                        match op {
                            UpdateOperator::Increment => self.emit(Instruction::Add),
                            UpdateOperator::Decrement => self.emit(Instruction::Sub),
                        }
                        self.emit(Instruction::StoreGlobal(name.clone()));
                    }
                } else if let Expression::Member {
                    object,
                    property,
                    computed,
                } = operand.as_ref()
                {
                    // Read old_value
                    self.generate_expression(object)?;
                    if *computed {
                        self.generate_expression(property)?;
                    } else if let Expression::Identifier(name) = property.as_ref() {
                        let idx = self.add_constant(Value::String(name.clone()));
                        self.emit(Instruction::LoadConst(idx));
                    } else {
                        self.generate_expression(property)?;
                    }
                    self.emit(Instruction::GetProperty);

                    if *prefix {
                        // ++this.count: result is new_value
                        let one = self.add_constant(Value::Float(1.0));
                        self.emit(Instruction::LoadConst(one));
                        match op {
                            UpdateOperator::Increment => self.emit(Instruction::Add),
                            UpdateOperator::Decrement => self.emit(Instruction::Sub),
                        }
                        // Stack: [new_value]
                        self.emit(Instruction::Dup);
                        self.generate_expression(object)?;
                        if *computed {
                            self.generate_expression(property)?;
                        } else if let Expression::Identifier(name) = property.as_ref() {
                            let idx = self.add_constant(Value::String(name.clone()));
                            self.emit(Instruction::LoadConst(idx));
                        } else {
                            self.generate_expression(property)?;
                        }
                        self.emit(Instruction::Rot3Right);
                        self.emit(Instruction::SetProperty);
                        self.emit(Instruction::Pop);
                    } else {
                        // this.count++: result is old_value
                        self.emit(Instruction::Dup);
                        let one = self.add_constant(Value::Float(1.0));
                        self.emit(Instruction::LoadConst(one));
                        match op {
                            UpdateOperator::Increment => self.emit(Instruction::Add),
                            UpdateOperator::Decrement => self.emit(Instruction::Sub),
                        }
                        // Stack: [old_value, new_value]
                        self.generate_expression(object)?;
                        if *computed {
                            self.generate_expression(property)?;
                        } else if let Expression::Identifier(name) = property.as_ref() {
                            let idx = self.add_constant(Value::String(name.clone()));
                            self.emit(Instruction::LoadConst(idx));
                        } else {
                            self.generate_expression(property)?;
                        }
                        self.emit(Instruction::Rot3Right);
                        self.emit(Instruction::SetProperty);
                        self.emit(Instruction::Pop);
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
                    self.emit(Instruction::LoadConst(idx));
                } else {
                    let first = &quasis[0];
                    if !first.is_empty() {
                        let idx = self.add_constant(Value::String(first.clone()));
                        self.emit(Instruction::LoadConst(idx));
                    }

                    for i in 0..expressions.len() {
                        if first.is_empty() && i == 0 {
                            self.generate_expression(&expressions[i])?;
                            self.emit(Instruction::ToString);
                        } else {
                            self.generate_expression(&expressions[i])?;
                            self.emit(Instruction::ToString);
                            self.emit(Instruction::Add);
                        }

                        if i + 1 < quasis.len() && !quasis[i + 1].is_empty() {
                            let idx = self.add_constant(Value::String(quasis[i + 1].clone()));
                            self.emit(Instruction::LoadConst(idx));
                            self.emit(Instruction::Add);
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

                self.emit(Instruction::MakeClass(class_info_idx));
                Ok(())
            }
            Expression::AwaitExpression { argument } => {
                self.generate_expression(argument)?;
                self.emit(Instruction::Await);
                Ok(())
            }
            Expression::ImportExpression { source } => {
                self.generate_expression(source)?;
                self.emit(Instruction::DynamicImport);
                Ok(())
            }
            Expression::SuperCall { args } => {
                self.emit(Instruction::LoadThis);
                for arg in args {
                    self.generate_expression(arg)?;
                }
                self.emit(Instruction::SuperConstruct(args.len() as u16));
                Ok(())
            }
            Expression::SuperMember { property, computed } => {
                self.emit(Instruction::LoadThis);
                if *computed {
                    self.generate_expression(property)?;
                } else if let Expression::Identifier(name) = property.as_ref() {
                    let idx = self.add_constant(Value::String(name.clone()));
                    self.emit(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.emit(Instruction::SuperGet);
                Ok(())
            }
            Expression::ArrayLiteral { elements } => {
                let has_spread = elements
                    .iter()
                    .any(|e| matches!(e, Expression::SpreadElement { .. }));
                if has_spread {
                    self.emit(Instruction::NewArray(0));
                    for elem in elements {
                        match elem {
                            Expression::SpreadElement { argument } => {
                                self.emit(Instruction::Dup);
                                self.generate_expression(argument)?;
                                self.emit(Instruction::SpreadArray);
                            }
                            _ => {
                                self.emit(Instruction::Dup);
                                self.generate_expression(elem)?;
                                self.emit(Instruction::ArrayPush);
                            }
                        }
                    }
                } else {
                    for elem in elements.iter().rev() {
                        self.generate_expression(elem)?;
                    }
                    self.emit(Instruction::NewArray(elements.len() as u32));
                }
                Ok(())
            }
            Expression::ObjectLiteral { properties } => {
                let has_spread = properties.iter().any(|p| p.key.is_empty());
                if has_spread {
                    self.emit(Instruction::NewObject);
                    for prop in properties {
                        if prop.key.is_empty()
                            && matches!(prop.value, Expression::SpreadElement { .. })
                        {
                            if let Expression::SpreadElement { argument } = &prop.value {
                                self.emit(Instruction::Dup);
                                self.generate_expression(argument)?;
                                self.emit(Instruction::SpreadObject);
                            }
                        } else {
                            if prop.computed {
                                if let Some(key_expr) = &prop.computed_key {
                                    self.generate_expression(key_expr)?;
                                }
                            } else {
                                let actual_key = if prop.is_getter {
                                    format!("__getter_{}", prop.key)
                                } else if prop.is_setter {
                                    format!("__setter_{}", prop.key)
                                } else {
                                    prop.key.clone()
                                };
                                let key_idx = self.add_constant(Value::String(actual_key));
                                self.emit(Instruction::LoadConst(key_idx));
                            }
                            self.generate_expression(&prop.value)?;
                            self.emit(Instruction::SetProperty);
                        }
                    }
                } else {
                    self.emit(Instruction::NewObject);
                    for prop in properties {
                        if prop.computed {
                            if let Some(key_expr) = &prop.computed_key {
                                self.generate_expression(key_expr)?;
                                self.emit(Instruction::ToString);
                            }
                        } else {
                            let actual_key = if prop.is_getter {
                                format!("__getter_{}", prop.key)
                            } else if prop.is_setter {
                                format!("__setter_{}", prop.key)
                            } else {
                                prop.key.clone()
                            };
                            let key_idx = self.add_constant(Value::String(actual_key));
                            self.emit(Instruction::LoadConst(key_idx));
                        }
                        self.generate_expression(&prop.value)?;
                        self.emit(Instruction::SetProperty);
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
            BinaryOperator::Add => self.emit(Instruction::Add),
            BinaryOperator::Sub => self.emit(Instruction::Sub),
            BinaryOperator::Mul => self.emit(Instruction::Mul),
            BinaryOperator::Div => self.emit(Instruction::Div),
            BinaryOperator::Mod => self.emit(Instruction::Mod),
            BinaryOperator::Power => self.emit(Instruction::Power),
            BinaryOperator::Eq => self.emit(Instruction::Eq),
            BinaryOperator::StrictEq => self.emit(Instruction::StrictEq),
            BinaryOperator::NotEqual => self.emit(Instruction::NotEqual),
            BinaryOperator::StrictNotEqual => self.emit(Instruction::StrictNotEqual),
            BinaryOperator::Less => self.emit(Instruction::Less),
            BinaryOperator::Greater => self.emit(Instruction::Greater),
            BinaryOperator::LessEqual => self.emit(Instruction::LessEqual),
            BinaryOperator::GreaterEqual => self.emit(Instruction::GreaterEqual),
            BinaryOperator::And => self.emit(Instruction::And),
            BinaryOperator::Or => self.emit(Instruction::Or),
            BinaryOperator::BitAnd => self.emit(Instruction::BitAnd),
            BinaryOperator::BitOr => self.emit(Instruction::BitOr),
            BinaryOperator::BitXor => self.emit(Instruction::BitXor),
            BinaryOperator::ShiftLeft => self.emit(Instruction::ShiftLeft),
            BinaryOperator::ShiftRight => self.emit(Instruction::ShiftRight),
            BinaryOperator::Instanceof => self.emit(Instruction::InstanceOf),
            BinaryOperator::In => self.emit(Instruction::In),
            BinaryOperator::NullishCoalescing => self.emit(Instruction::NullishCoalescing),
            BinaryOperator::Comma => {
                self.emit(Instruction::Pop);
            }
        }
        Ok(())
    }
}
