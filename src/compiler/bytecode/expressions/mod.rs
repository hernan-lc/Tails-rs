mod calls;
mod literals;
mod operators;

use super::*;
use crate::errors::Result;

impl CodeGenerator {
    pub(crate) fn generate_expression(&mut self, expr: &Expression) -> Result<()> {
        match expr {
            Expression::NumberLiteral(n) => self.generate_number_literal(*n),
            Expression::BigIntLiteral(s) => self.generate_bigint_literal(s),
            Expression::StringLiteral(s) => self.generate_string_literal(s),
            Expression::RegexLiteral { pattern, flags } => {
                self.generate_regex_literal(pattern, flags)
            }
            Expression::BooleanLiteral(b) => self.generate_boolean_literal(*b),
            Expression::NullLiteral => {
                self.emit(Instruction::LoadNull);
                Ok(())
            }
            Expression::UndefinedLiteral => {
                self.emit(Instruction::LoadUndefined);
                Ok(())
            }
            Expression::NaNLiteral => self.generate_nan_literal(),
            Expression::InfinityLiteral => self.generate_infinity_literal(),
            Expression::Identifier(name) => self.generate_identifier(name),
            Expression::BinaryOp { op, left, right } => {
                self.generate_binary_op_expr(op, left, right)
            }
            Expression::UnaryOp { op, operand } => self.generate_unary_op(op, operand),
            Expression::Assignment { target, value, op } => {
                self.generate_assignment(target, value, op)
            }
            Expression::Call { callee, args } => self.generate_call(callee, args),
            Expression::Member {
                object,
                property,
                computed,
            } => self.generate_member(object, property, *computed),
            Expression::OptionalMember {
                object,
                property,
                computed,
            } => self.generate_optional_member(object, property, *computed),
            Expression::OptionalCall { callee, args } => self.generate_optional_call(callee, args),
            Expression::FunctionExpression {
                name: _,
                params,
                param_patterns,
                body,
                is_async: _,
                param_types: _,
                return_type: _,
                is_generator,
                defaults,
                rest_param,
            } => self.generate_function_expression(
                params,
                body,
                *is_generator,
                rest_param.as_deref(),
                defaults,
                param_patterns,
            ),
            Expression::ArrowFunction {
                params,
                param_patterns,
                body,
                is_async: _,
                param_types: _,
                return_type: _,
                defaults,
                rest_param,
            } => self.generate_arrow_function(
                params,
                body,
                rest_param.as_deref(),
                defaults,
                param_patterns,
            ),
            Expression::NewExpression { callee, args } => {
                self.generate_new_expression(callee, args)
            }
            Expression::ConditionalExpression {
                test,
                consequent,
                alternate,
            } => self.generate_conditional(test, consequent, alternate),
            Expression::UpdateExpression {
                op,
                operand,
                prefix,
            } => self.generate_update(op, operand, *prefix),
            Expression::TemplateLiteral {
                quasis,
                expressions,
            } => self.generate_template_literal(quasis, expressions),
            Expression::ClassExpression {
                name,
                superclass,
                body,
            } => self.generate_class_expression(name, superclass, body),
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
            Expression::SuperCall { args } => self.generate_super_call(args),
            Expression::SuperMember { property, computed } => {
                self.generate_super_member(property, *computed)
            }
            Expression::ArrayLiteral { elements } => self.generate_array_literal(elements),
            Expression::ObjectLiteral { properties } => self.generate_object_literal(properties),
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

    pub(crate) fn generate_binary_op(&mut self, op: &BinaryOperator) -> Result<()> {
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

    fn generate_member(
        &mut self,
        object: &Expression,
        property: &Expression,
        computed: bool,
    ) -> Result<()> {
        self.generate_expression(object)?;
        if computed {
            self.generate_expression(property)?;
        } else if let Expression::Identifier(name) = property {
            let idx = self.add_constant(Value::from_string(name.clone()));
            self.emit(Instruction::LoadConst(idx));
        } else {
            self.generate_expression(property)?;
        }
        self.emit(Instruction::GetProperty);
        Ok(())
    }

    fn generate_optional_member(
        &mut self,
        object: &Expression,
        property: &Expression,
        computed: bool,
    ) -> Result<()> {
        self.generate_expression(object)?;
        self.emit(Instruction::Dup);
        let check_undef = self.instructions.len();
        self.emit(Instruction::JumpIfUndefined(0));
        if computed {
            self.generate_expression(property)?;
        } else if let Expression::Identifier(name) = property {
            let idx = self.add_constant(Value::from_string(name.clone()));
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

    fn emit_param_destructuring(
        &mut self,
        params: &[String],
        param_patterns: &[Option<crate::compiler::parser::BindingPattern>],
    ) -> Result<()> {
        for (i, pattern_opt) in param_patterns.iter().enumerate() {
            if let Some(pattern) = pattern_opt {
                // Register pattern binding names as locals before destructuring.
                let mut names = Vec::new();
                super::stmt_function::collect_binding_names(pattern, &mut names);
                for name in &names {
                    if !self.locals.iter().any(|l| l == name) {
                        self.locals.push(name.clone());
                    }
                }
                let param_slot = self
                    .resolve_local(&params[i])
                    .unwrap_or_else(|| self.last_local_slot());
                self.emit(Instruction::LoadLocal(param_slot));
                self.generate_destructuring_pattern(pattern)?;
            }
        }
        Ok(())
    }

    fn generate_function_expression(
        &mut self,
        params: &[String],
        body: &[SpannedNode<Statement>],
        is_generator: bool,
        rest_param: Option<&str>,
        defaults: &[Option<Expression>],
        param_patterns: &[Option<crate::compiler::parser::BindingPattern>],
    ) -> Result<()> {
        let func_idx = self.functions.len() as u32;
        let mut all_params = params.to_vec();
        if let Some(rp) = rest_param {
            all_params.push(rp.to_string());
        }
        let outer_refs = super::closures::find_outer_refs_with_slots(body, &all_params, |name| {
            self.resolve_local(name)
        });
        let num_captures = outer_refs.len();

        self.functions.push(CompiledFunction {
            name: None,
            params: params.to_vec(),
            rest_param: rest_param.map(|s| s.to_string()),
            bytecode_index: 0,
            param_count: params.len(),
            closure_var_count: num_captures,
            local_count: 0,
            is_generator,
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
            self.locals.push(rp.to_string());
        }

        self.compile_default_params(params, defaults)?;
        self.emit_param_destructuring(params, param_patterns)?;

        for stmt in body {
            self.record_line_from_span(&stmt.span);
            self.generate_statement(&stmt.inner, false)?;
        }

        self.emit(Instruction::LoadUndefined);
        self.emit(Instruction::Return);

        self.finalize_local_count(func_idx);

        self.scope_depth -= 1;
        self.locals.truncate(prev_locals);
        self.captured_var_names = saved_captured;
        self.local_start_idx = saved_start;

        self.patch_jump(jump_over, self.instructions.len());

        if num_captures > 0 {
            let capture_slots: Vec<u16> = outer_refs.iter().map(|(_, s)| *s).collect();
            self.emit(Instruction::MakeClosure(func_idx, Box::new(capture_slots)));
        } else {
            self.emit(Instruction::MakeFunction(func_idx));
        }
        Ok(())
    }

    fn generate_arrow_function(
        &mut self,
        params: &[String],
        body: &ArrowFunctionBody,
        rest_param: Option<&str>,
        defaults: &[Option<Expression>],
        param_patterns: &[Option<crate::compiler::parser::BindingPattern>],
    ) -> Result<()> {
        let func_idx = self.functions.len() as u32;

        let (body_stmts, is_expr) = match body {
            ArrowFunctionBody::Expression(expr) => (
                vec![SpannedNode {
                    inner: Statement::ReturnStatement(Some(expr.clone())),
                    span: None,
                }],
                true,
            ),
            ArrowFunctionBody::Block(stmts) => (stmts.clone(), false),
        };

        let mut all_params = params.to_vec();
        if let Some(rp) = rest_param {
            all_params.push(rp.to_string());
        }
        let outer_refs =
            super::closures::find_outer_refs_with_slots(&body_stmts, &all_params, |name| {
                self.resolve_local(name)
            });
        let num_captures = outer_refs.len();

        self.functions.push(CompiledFunction {
            name: None,
            params: params.to_vec(),
            rest_param: rest_param.map(|s| s.to_string()),
            bytecode_index: 0,
            param_count: params.len(),
            closure_var_count: num_captures,
            local_count: 0,
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
            self.locals.push(rp.to_string());
        }

        self.compile_default_params(params, defaults)?;
        self.emit_param_destructuring(params, param_patterns)?;

        for stmt in &body_stmts {
            self.record_line_from_span(&stmt.span);
            self.generate_statement(&stmt.inner, false)?;
        }

        if !is_expr {
            self.emit(Instruction::LoadUndefined);
            self.emit(Instruction::Return);
        }

        self.finalize_local_count(func_idx);

        self.scope_depth -= 1;
        self.locals.truncate(prev_locals);
        self.captured_var_names = saved_captured;
        self.local_start_idx = saved_start;

        self.patch_jump(jump_over, self.instructions.len());

        if num_captures > 0 {
            let capture_slots: Vec<u16> = outer_refs.iter().map(|(_, s)| *s).collect();
            self.emit(Instruction::MakeClosure(func_idx, Box::new(capture_slots)));
        } else {
            self.emit(Instruction::MakeFunction(func_idx));
        }
        Ok(())
    }

    fn generate_class_expression(
        &mut self,
        name: &Option<String>,
        superclass: &Option<Box<Expression>>,
        body: &[ClassMember],
    ) -> Result<()> {
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
                    let func_idx =
                        self.compile_function(Some(format!("get_{}", mname)), &[], mbody, false)?;
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

    fn generate_super_call(&mut self, args: &[Expression]) -> Result<()> {
        self.emit(Instruction::LoadThis);
        for arg in args {
            self.generate_expression(arg)?;
        }
        self.emit(Instruction::SuperConstruct(args.len() as u16));
        Ok(())
    }

    fn generate_super_member(&mut self, property: &Expression, computed: bool) -> Result<()> {
        self.emit(Instruction::LoadThis);
        if computed {
            self.generate_expression(property)?;
        } else if let Expression::Identifier(name) = property {
            let idx = self.add_constant(Value::from_string(name.clone()));
            self.emit(Instruction::LoadConst(idx));
        } else {
            self.generate_expression(property)?;
        }
        self.emit(Instruction::SuperGet);
        Ok(())
    }

    fn generate_array_literal(&mut self, elements: &[Expression]) -> Result<()> {
        let has_spread = elements
            .iter()
            .any(|e| matches!(e, Expression::SpreadElement { .. }));
        if has_spread {
            self.emit(Instruction::NewArray(0));
            for elem in elements {
                match elem {
                    Expression::SpreadElement { argument } => {
                        self.generate_expression(argument)?;
                        self.emit(Instruction::SpreadArray);
                    }
                    _ => {
                        // ArrayPush pops (array, value) and pushes the array back.
                        // Do NOT Dup first — that leaves a stale array slot under
                        // the result and corrupts CallMethod stacks.
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

    fn generate_object_literal(
        &mut self,
        properties: &[crate::compiler::parser::ObjectProperty],
    ) -> Result<()> {
        let has_spread = properties
            .iter()
            .any(|p| matches!(p.value, Expression::SpreadElement { .. }));
        if has_spread {
            self.emit(Instruction::NewObject);
            for prop in properties {
                if matches!(prop.value, Expression::SpreadElement { .. }) {
                    if let Expression::SpreadElement { argument } = &prop.value {
                        self.generate_expression(argument)?;
                        self.emit(Instruction::SpreadObject);
                    }
                } else if prop.computed {
                    if let Some(key_expr) = &prop.computed_key {
                        self.generate_expression(key_expr)?;
                    }
                    self.generate_expression(&prop.value)?;
                    self.emit(Instruction::SetProperty);
                } else {
                    let actual_key = if prop.is_getter {
                        format!("__getter_{}", prop.key)
                    } else if prop.is_setter {
                        format!("__setter_{}", prop.key)
                    } else {
                        prop.key.clone()
                    };
                    let key_idx = self.add_constant(Value::from_string(actual_key));
                    self.emit(Instruction::LoadConst(key_idx));
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
                    }
                } else {
                    let actual_key = if prop.is_getter {
                        format!("__getter_{}", prop.key)
                    } else if prop.is_setter {
                        format!("__setter_{}", prop.key)
                    } else {
                        prop.key.clone()
                    };
                    let key_idx = self.add_constant(Value::from_string(actual_key));
                    self.emit(Instruction::LoadConst(key_idx));
                }
                self.generate_expression(&prop.value)?;
                self.emit(Instruction::SetProperty);
            }
        }
        Ok(())
    }
}
