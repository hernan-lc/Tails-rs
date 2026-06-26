use std::collections::HashSet;
use crate::compiler::parser::{AstNode, Statement, Expression, BinaryOperator, UnaryOperator, ForInit, ForInLeft, CompoundAssignmentOp, UpdateOperator, ArrowFunctionBody, ClassMember};
use crate::compiler::{CompiledModule, CompiledFunction, Instruction, ClassInfo, ClassMethodInfo, ClassMethodKind};
use crate::errors::Result;
use crate::objects::Value;

pub fn generate(ast: &AstNode) -> Result<CompiledModule> {
    let mut generator = CodeGenerator::new();
    generator.generate(ast)
}

struct CodeGenerator {
    constants: Vec<Value>,
    instructions: Vec<Instruction>,
    functions: Vec<CompiledFunction>,
    locals: Vec<String>,
    scope_depth: usize,
    captured_var_names: Vec<String>,
    local_start_idx: usize,
    break_targets: Vec<usize>,
    continue_targets: Vec<usize>,
    class_infos: Vec<ClassInfo>,
}

impl CodeGenerator {
    fn new() -> Self {
        Self {
            constants: Vec::new(),
            instructions: Vec::new(),
            functions: Vec::new(),
            locals: Vec::new(),
            scope_depth: 0,
            captured_var_names: Vec::new(),
            local_start_idx: 0,
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
            class_infos: Vec::new(),
        }
    }

    fn generate(&mut self, ast: &AstNode) -> Result<CompiledModule> {
        match ast {
            AstNode::Program(statements) => {
                for (i, stmt) in statements.iter().enumerate() {
                    let is_last = i == statements.len() - 1;
                    self.generate_statement(stmt, is_last)?;
                }
                if statements.is_empty() {
                    self.instructions.push(Instruction::LoadUndefined);
                }
            }
            _ => return Err(crate::errors::Error::InternalError("Invalid AST node".into())),
        }

        Ok(CompiledModule {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
            functions: self.functions.clone(),
            class_infos: self.class_infos.clone(),
        })
    }

    fn generate_statement(&mut self, stmt: &Statement, is_last: bool) -> Result<()> {
        match stmt {
            Statement::Expression(expr) => {
                let is_assignment = matches!(expr, Expression::Assignment { .. });
                self.generate_expression(expr)?;
                if !is_last && !is_assignment {
                    self.instructions.push(Instruction::Pop);
                }
                Ok(())
            }
            Statement::VariableDeclaration { kind: _, declarations } => {
                for decl in declarations {
                    if let Some(init) = &decl.init {
                        self.generate_expression(init)?;
                    } else {
                        self.instructions.push(Instruction::LoadUndefined);
                    }

                    if self.scope_depth == 0 {
                        self.instructions.push(Instruction::StoreGlobal(decl.id.clone()));
                    } else {
                        self.locals.push(decl.id.clone());
                        let slot = (self.locals.len() - 1) as u16;
                        self.instructions.push(Instruction::StoreLocal(slot));
                    }
                }
                Ok(())
            }
            Statement::ReturnStatement(value) => {
                if let Some(expr) = value {
                    self.generate_expression(expr)?;
                } else {
                    self.instructions.push(Instruction::LoadUndefined);
                }
                self.instructions.push(Instruction::Return);
                Ok(())
            }
            Statement::IfStatement { condition, consequent, alternate } => {
                self.generate_expression(condition)?;
                let jump_if_not = self.instructions.len();
                self.instructions.push(Instruction::JumpIfNot(0));
                self.generate_statement(consequent, false)?;
                if let Some(alt) = alternate {
                    let jump_to_end = self.instructions.len();
                    self.instructions.push(Instruction::Jump(0));
                    self.patch_jump(jump_if_not, self.instructions.len());
                    self.generate_statement(alt, false)?;
                    self.patch_jump(jump_to_end, self.instructions.len());
                } else {
                    self.patch_jump(jump_if_not, self.instructions.len());
                }
                Ok(())
            }
            Statement::WhileStatement { condition, body } => {
                let loop_start = self.instructions.len() as u32;
                self.generate_expression(condition)?;
                let jump_if_not = self.instructions.len();
                self.instructions.push(Instruction::JumpIfNot(0));
                self.continue_targets.push(loop_start as usize);
                let break_start = self.break_targets.len();
                self.break_targets.push(usize::MAX);
                self.generate_statement(body, false)?;
                self.instructions.push(Instruction::Jump(loop_start));
                let loop_end = self.instructions.len();
                self.patch_jump(jump_if_not, loop_end);
                while self.break_targets.len() > break_start {
                    let idx = self.break_targets.pop().unwrap();
                    self.patch_jump(idx, loop_end);
                }
                self.continue_targets.pop();
                Ok(())
            }
            Statement::ForStatement { init, condition, update, body } => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();

                if let Some(for_init) = init {
                    match for_init.as_ref() {
                        ForInit::Variable(stmt) => {
                            self.generate_statement(stmt, false)?;
                        }
                        ForInit::Expression(expr) => {
                            self.generate_expression(expr)?;
                            self.instructions.push(Instruction::Pop);
                        }
                    }
                }

                let loop_start = self.instructions.len() as u32;

                let jump_to_end = if let Some(cond) = condition {
                    self.generate_expression(cond)?;
                    let j = self.instructions.len();
                    self.instructions.push(Instruction::JumpIfNot(0));
                    Some(j)
                } else {
                    None
                };

                self.continue_targets.push(loop_start as usize);
                let break_start = self.break_targets.len();
                self.break_targets.push(usize::MAX);
                self.generate_statement(body, false)?;

                if let Some(upd) = update {
                    let is_assignment = matches!(upd, Expression::Assignment { .. });
                    self.generate_expression(upd)?;
                    if !is_assignment {
                        self.instructions.push(Instruction::Pop);
                    }
                }

                self.instructions.push(Instruction::Jump(loop_start));

                let loop_end = self.instructions.len();
                while self.break_targets.len() > break_start {
                    let idx = self.break_targets.pop().unwrap();
                    self.patch_jump(idx, loop_end);
                }
                self.continue_targets.pop();

                if let Some(j) = jump_to_end {
                    self.patch_jump(j, self.instructions.len());
                }

                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                    self.instructions.push(Instruction::Pop);
                }
                self.scope_depth -= 1;
                Ok(())
            }
            Statement::ForInStatement { left, right, body } => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();

                let var_name = match left {
                    ForInLeft::Identifier(id) => id.clone(),
                    ForInLeft::VariableDeclaration { id, .. } => id.clone(),
                };

                self.locals.push(var_name.clone());
                let var_slot = (self.locals.len() - 1) as u16;

                self.generate_expression(right)?;
                let iter_slot = self.locals.len() as u16;
                self.locals.push("__iter".to_string());
                self.instructions.push(Instruction::StoreLocal(iter_slot));

                let idx_slot = self.locals.len() as u16;
                self.locals.push("__idx".to_string());
                let zero_idx = self.add_constant(Value::Float(0.0));
                self.instructions.push(Instruction::LoadConst(zero_idx));
                self.instructions.push(Instruction::StoreLocal(idx_slot));

                let loop_start = self.instructions.len() as u32;

                self.instructions.push(Instruction::LoadLocal(iter_slot));
                self.instructions.push(Instruction::LoadLocal(idx_slot));
                self.instructions.push(Instruction::GetProperty);
                self.instructions.push(Instruction::TypeOf);
                let undef_idx = self.add_constant(Value::String("undefined".to_string()));
                self.instructions.push(Instruction::LoadConst(undef_idx));
                self.instructions.push(Instruction::StrictEq);
                let jump_if_done = self.instructions.len();
                self.instructions.push(Instruction::JumpIf(0));

                self.instructions.push(Instruction::LoadLocal(iter_slot));
                self.instructions.push(Instruction::LoadLocal(idx_slot));
                self.instructions.push(Instruction::GetProperty);
                self.instructions.push(Instruction::StoreLocal(var_slot));

                self.generate_statement(body, false)?;

                self.instructions.push(Instruction::LoadLocal(idx_slot));
                let one_idx = self.add_constant(Value::Float(1.0));
                self.instructions.push(Instruction::LoadConst(one_idx));
                self.instructions.push(Instruction::Add);
                self.instructions.push(Instruction::StoreLocal(idx_slot));

                self.instructions.push(Instruction::Jump(loop_start));

                self.patch_jump(jump_if_done, self.instructions.len());

                self.locals.pop();
                self.locals.pop();
                self.locals.pop();

                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                    self.instructions.push(Instruction::Pop);
                }
                self.scope_depth -= 1;
                Ok(())
            }
            Statement::ForOfStatement { left, right, body, is_async: _ } => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();

                let var_name = match left {
                    ForInLeft::Identifier(id) => id.clone(),
                    ForInLeft::VariableDeclaration { id, .. } => id.clone(),
                };

                self.instructions.push(Instruction::LoadUndefined);
                self.locals.push(var_name.clone());
                let var_slot = (self.locals.len() - 1) as u16;

                self.generate_expression(right)?;

                let iter_slot = self.locals.len() as u16;
                self.locals.push("__iter".to_string());
                self.instructions.push(Instruction::StoreLocal(iter_slot));

                let idx_slot = self.locals.len() as u16;
                self.locals.push("__idx".to_string());
                let zero_idx = self.add_constant(Value::Float(0.0));
                self.instructions.push(Instruction::LoadConst(zero_idx));
                self.instructions.push(Instruction::StoreLocal(idx_slot));

                let loop_start = self.instructions.len() as u32;

                self.instructions.push(Instruction::LoadLocal(idx_slot));
                let jump_if_done = self.instructions.len();
                self.instructions.push(Instruction::JumpIfNot(0));

                self.instructions.push(Instruction::LoadLocal(iter_slot));
                self.instructions.push(Instruction::LoadLocal(idx_slot));
                self.instructions.push(Instruction::GetProperty);
                self.instructions.push(Instruction::StoreLocal(var_slot));

                self.generate_statement(body, false)?;

                self.instructions.push(Instruction::LoadLocal(idx_slot));
                let one_idx = self.add_constant(Value::Float(1.0));
                self.instructions.push(Instruction::LoadConst(one_idx));
                self.instructions.push(Instruction::Add);
                self.instructions.push(Instruction::StoreLocal(idx_slot));

                self.instructions.push(Instruction::Jump(loop_start));

                self.patch_jump(jump_if_done, self.instructions.len());

                self.locals.pop();
                self.locals.pop();

                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                    self.instructions.push(Instruction::Pop);
                }
                self.scope_depth -= 1;
                Ok(())
            }
            Statement::DoWhileStatement { condition, body } => {
                let loop_start = self.instructions.len() as u32;

                let break_start = self.break_targets.len();
                self.break_targets.push(usize::MAX);
                self.generate_statement(body, false)?;

                self.generate_expression(condition)?;

                let jump_if_not = self.instructions.len();
                self.instructions.push(Instruction::JumpIfNot(0));

                self.instructions.push(Instruction::Jump(loop_start));

                let loop_end = self.instructions.len();
                self.patch_jump(jump_if_not, loop_end);
                while self.break_targets.len() > break_start {
                    let idx = self.break_targets.pop().unwrap();
                    self.patch_jump(idx, loop_end);
                }

                Ok(())
            }
            Statement::SwitchStatement { discriminant, cases } => {
                self.generate_expression(discriminant)?;

                self.locals.push("__switch_val".to_string());
                let disc_slot = (self.locals.len() - 1) as u16;
                self.instructions.push(Instruction::StoreLocal(disc_slot));

                let mut body_jumps: Vec<usize> = Vec::new();
                let mut default_jump: Option<usize> = None;

                for case in cases {
                    if let Some(test) = &case.test {
                        self.instructions.push(Instruction::LoadLocal(disc_slot));
                        self.generate_expression(test)?;
                        self.instructions.push(Instruction::StrictEq);
                        let j = self.instructions.len();
                        self.instructions.push(Instruction::JumpIf(0));
                        body_jumps.push(j);
                    } else {
                        default_jump = Some(self.instructions.len());
                        self.instructions.push(Instruction::Jump(0));
                    }
                }

                let end_jump = self.instructions.len();
                self.instructions.push(Instruction::Jump(0));

                let break_start = self.break_targets.len();
                let mut non_default_idx = 0;
                for case in cases {
                    if case.test.is_some() {
                        if !body_jumps.is_empty() {
                            let jmp_idx = body_jumps.remove(0);
                            self.patch_jump(jmp_idx, self.instructions.len());
                        }
                    } else if let Some(j) = default_jump.take() {
                        self.patch_jump(j, self.instructions.len());
                    }
                    for s in &case.consequent {
                        self.generate_statement(s, false)?;
                    }
                    let _ = non_default_idx;
                }

                let loop_end = self.instructions.len();
                while self.break_targets.len() > break_start {
                    let idx = self.break_targets.pop().unwrap();
                    self.patch_jump(idx, loop_end);
                }

                self.patch_jump(end_jump, loop_end);

                self.locals.pop();
                self.instructions.push(Instruction::Pop);

                Ok(())
            }
            Statement::BreakStatement => {
                if !self.break_targets.is_empty() {
                    self.instructions.push(Instruction::Jump(0));
                    self.break_targets.push(self.instructions.len() - 1);
                } else {
                    self.instructions.push(Instruction::LoadUndefined);
                    self.instructions.push(Instruction::Return);
                }
                Ok(())
            }
            Statement::ContinueStatement => {
                if let Some(target) = self.continue_targets.last().copied() {
                    self.instructions.push(Instruction::Jump(target as u32));
                } else {
                    self.instructions.push(Instruction::LoadUndefined);
                    self.instructions.push(Instruction::Return);
                }
                Ok(())
            }
            Statement::TryStatement { block, handler, finalizer } => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();

                let has_catch = handler.is_some();
                let has_finally = finalizer.is_some();

                self.instructions.push(Instruction::TryJump(0, 0));
                let try_jump_idx = self.instructions.len() - 1;

                for stmt in block {
                    self.generate_statement(stmt, false)?;
                }

                self.instructions.push(Instruction::PopTryHandler);

                let jump_past_catch = if has_catch {
                    let j = self.instructions.len();
                    self.instructions.push(Instruction::Jump(0));
                    Some(j)
                } else {
                    None
                };

                let catch_pc = if has_catch {
                    self.instructions.len() as u32
                } else {
                    0
                };

                if let Some(catch_clause) = handler {
                    self.locals.push(catch_clause.param.clone());
                    let slot = (self.locals.len() - 1) as u16;
                    self.instructions.push(Instruction::LoadException);
                    self.instructions.push(Instruction::StoreLocal(slot));
                    let last_idx = catch_clause.body.len().saturating_sub(1);
                    for (i, stmt) in catch_clause.body.iter().enumerate() {
                        self.generate_statement(stmt, i == last_idx && !has_finally)?;
                    }
                }

                let finally_pc = self.instructions.len() as u32;

                if let Some(j) = jump_past_catch {
                    self.patch_jump(j, finally_pc as usize);
                }

                if let Instruction::TryJump(ref mut c, ref mut f) = self.instructions[try_jump_idx] {
                    *c = catch_pc;
                    *f = finally_pc;
                }

                if has_finally {
                    for stmt in finalizer.as_ref().unwrap() {
                        self.generate_statement(stmt, false)?;
                    }
                    if !has_catch {
                        self.instructions.push(Instruction::ReThrowIfPending);
                    }
                }

                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                    self.instructions.push(Instruction::Pop);
                }
                self.scope_depth -= 1;
                Ok(())
            }
            Statement::ThrowStatement(argument) => {
                self.generate_expression(argument)?;
                self.instructions.push(Instruction::Throw);
                Ok(())
            }
            Statement::ClassDeclaration { name, superclass, body } => {
                let class_name = name.clone();

                let class_info_idx = self.class_infos.len() as u32;

                let constructor_func_idx = self.compile_class_constructor(body)?;

                let mut methods = Vec::new();
                for member in body {
                    match member {
                        ClassMember::Method { name: mname, params, body: mbody, is_static, is_async: _ } => {
                            let func_idx = self.compile_function(Some(mname.clone()), params, mbody)?;
                            methods.push(ClassMethodInfo {
                                name: mname.clone(),
                                func_idx,
                                is_static: *is_static,
                                kind: ClassMethodKind::Method,
                            });
                        }
                        ClassMember::Getter { name: mname, body: mbody, is_static } => {
                            let func_idx = self.compile_function(Some(format!("get_{}", mname)), &vec![], mbody)?;
                            methods.push(ClassMethodInfo {
                                name: mname.clone(),
                                func_idx,
                                is_static: *is_static,
                                kind: ClassMethodKind::Getter,
                            });
                        }
                        ClassMember::Setter { name: mname, param, body: mbody, is_static } => {
                            let func_idx = self.compile_function(Some(format!("set_{}", mname)), &vec![param.clone()], mbody)?;
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

                self.instructions.push(Instruction::MakeClass(class_info_idx));

                if self.scope_depth == 0 {
                    self.instructions.push(Instruction::StoreGlobal(class_name));
                } else {
                    self.locals.push(class_name);
                    let slot = (self.locals.len() - 1) as u16;
                    self.instructions.push(Instruction::StoreLocal(slot));
                }
                Ok(())
            }
            Statement::ImportDeclaration { source, .. } => {
                let _src_idx = self.add_constant(Value::String(source.clone()));
                Ok(())
            }
            Statement::ExportDeclaration { declaration } => {
                self.generate_statement(declaration, false)?;
                Ok(())
            }
            Statement::ExportDefaultDeclaration { declaration } => {
                self.generate_statement(declaration, false)?;
                Ok(())
            }
            Statement::BlockStatement(stmts) => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();
                for stmt in stmts {
                    self.generate_statement(stmt, false)?;
                }
                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                    self.instructions.push(Instruction::Pop);
                }
                self.scope_depth -= 1;
                Ok(())
            }
            Statement::FunctionDeclaration { name, params, body, is_async: _ } => {
                let func_idx = self.functions.len() as u32;
                let parent_locals_snapshot = self.locals.clone();
                let outer_refs = find_outer_refs(body, params, &parent_locals_snapshot);
                let num_captures = outer_refs.len();

                self.functions.push(CompiledFunction {
                    name: Some(name.clone()),
                    params: params.clone(),
                    bytecode_index: 0,
                    param_count: params.len(),
                    closure_var_count: num_captures,
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
                    self.instructions.push(Instruction::MakeClosure(func_idx, capture_slots));
                } else {
                    self.instructions.push(Instruction::MakeFunction(func_idx));
                }
                if self.scope_depth == 0 {
                    self.instructions.push(Instruction::StoreGlobal(name.clone()));
                } else {
                    self.locals.push(name.clone());
                    let slot = (self.locals.len() - 1) as u16;
                    self.instructions.push(Instruction::StoreLocal(slot));
                }
                Ok(())
            }
        }
    }

    fn generate_expression(&mut self, expr: &Expression) -> Result<()> {
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
            Expression::Identifier(name) => {
                if name == "this" {
                    self.instructions.push(Instruction::LoadThis);
                } else if let Some(local_idx) = self.resolve_local(name) {
                    self.instructions.push(Instruction::LoadLocal(local_idx));
                } else {
                    self.instructions.push(Instruction::LoadGlobal(name.clone()));
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
                        if let Expression::Member { object, property, computed } = operand.as_ref() {
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
                    UnaryOperator::Void if matches!(operand.as_ref(), Expression::Assignment { .. }) => {
                        self.generate_expression(operand)?;
                        self.instructions.push(Instruction::Pop);
                        self.instructions.push(Instruction::LoadUndefined);
                    }
                    _ => {
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
                            self.instructions.push(Instruction::StoreGlobal(name.clone()));
                        }
                    } else if let Expression::Member { object, property, computed } = target.as_ref() {
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
                        return Err(crate::errors::Error::RuntimeError("Invalid assignment target".into()));
                    }
                } else {
                    if let Expression::Member { object, property, computed } = target.as_ref() {
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
                            self.instructions.push(Instruction::StoreGlobal(name.clone()));
                        }
                    } else {
                        return Err(crate::errors::Error::RuntimeError("Invalid assignment target".into()));
                    }
                }
                Ok(())
            }
            Expression::Call { callee, args } => {
                if let Expression::Member { object, property, computed } = callee.as_ref() {
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
                    self.instructions.push(Instruction::CallMethod(args.len() as u16));
                } else {
                    for arg in args {
                        self.generate_expression(arg)?;
                    }
                    self.generate_expression(callee)?;
                    self.instructions.push(Instruction::Call(args.len() as u16));
                }
                Ok(())
            }
            Expression::Member { object, property, computed } => {
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
            Expression::FunctionExpression { name: _, params, body, is_async: _ } => {
                let func_idx = self.functions.len() as u32;
                let parent_locals_snapshot = self.locals.clone();
                let outer_refs = find_outer_refs(body, params, &parent_locals_snapshot);
                let num_captures = outer_refs.len();

                self.functions.push(CompiledFunction {
                    name: None,
                    params: params.clone(),
                    bytecode_index: 0,
                    param_count: params.len(),
                    closure_var_count: num_captures,
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
                    self.instructions.push(Instruction::MakeClosure(func_idx, capture_slots));
                } else {
                    self.instructions.push(Instruction::MakeFunction(func_idx));
                }
                Ok(())
            }
            Expression::ArrowFunction { params, body, is_async: _ } => {
                let func_idx = self.functions.len() as u32;

                let (body_stmts, is_expr) = match body.as_ref() {
                    ArrowFunctionBody::Expression(expr) => {
                        (vec![Statement::ReturnStatement(Some(expr.clone()))], true)
                    }
                    ArrowFunctionBody::Block(stmts) => {
                        (stmts.clone(), false)
                    }
                };

                let parent_locals_snapshot = self.locals.clone();
                let outer_refs = find_outer_refs(&body_stmts, params, &parent_locals_snapshot);
                let num_captures = outer_refs.len();

                self.functions.push(CompiledFunction {
                    name: None,
                    params: params.clone(),
                    bytecode_index: 0,
                    param_count: params.len(),
                    closure_var_count: num_captures,
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
                    self.instructions.push(Instruction::MakeClosure(func_idx, capture_slots));
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
                self.instructions.push(Instruction::Construct(args.len() as u16));
                Ok(())
            }
            Expression::ConditionalExpression { test, consequent, alternate } => {
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
            Expression::UpdateExpression { op, operand, prefix } => {
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
                            self.instructions.push(Instruction::StoreGlobal(name.clone()));
                        }
                    } else {
                        self.generate_expression(operand)?;
                        if let Some(local_idx) = self.resolve_local(name) {
                            self.instructions.push(Instruction::LoadLocal(local_idx));
                        } else {
                            self.instructions.push(Instruction::LoadGlobal(name.clone()));
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
                            self.instructions.push(Instruction::StoreGlobal(name.clone()));
                        }
                    }
                } else {
                    self.generate_expression(operand)?;
                }
                Ok(())
            }
            Expression::TemplateLiteral { quasis, expressions } => {
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
            Expression::ClassExpression { name, superclass, body } => {
                let class_info_idx = self.class_infos.len() as u32;
                let class_name = name.clone().unwrap_or_else(|| "anonymous".to_string());

                let constructor_func_idx = self.compile_class_constructor(body)?;

                let mut methods = Vec::new();
                for member in body {
                    match member {
                        ClassMember::Method { name: mname, params, body: mbody, is_static, is_async: _ } => {
                            let func_idx = self.compile_function(Some(mname.clone()), params, mbody)?;
                            methods.push(ClassMethodInfo {
                                name: mname.clone(),
                                func_idx,
                                is_static: *is_static,
                                kind: ClassMethodKind::Method,
                            });
                        }
                        ClassMember::Getter { name: mname, body: mbody, is_static } => {
                            let func_idx = self.compile_function(Some(format!("get_{}", mname)), &vec![], mbody)?;
                            methods.push(ClassMethodInfo {
                                name: mname.clone(),
                                func_idx,
                                is_static: *is_static,
                                kind: ClassMethodKind::Getter,
                            });
                        }
                        ClassMember::Setter { name: mname, param, body: mbody, is_static } => {
                            let func_idx = self.compile_function(Some(format!("set_{}", mname)), &vec![param.clone()], mbody)?;
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

                self.instructions.push(Instruction::MakeClass(class_info_idx));
                Ok(())
            }
            Expression::AwaitExpression { argument } => {
                self.generate_expression(argument)?;
                Ok(())
            }
            Expression::SuperCall { args } => {
                self.instructions.push(Instruction::LoadThis);
                for arg in args {
                    self.generate_expression(arg)?;
                }
                self.instructions.push(Instruction::SuperConstruct(args.len() as u16));
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
                for elem in elements.iter().rev() {
                    self.generate_expression(elem)?;
                }
                self.instructions.push(Instruction::NewArray(elements.len() as u32));
                Ok(())
            }
            Expression::ObjectLiteral { properties } => {
                self.instructions.push(Instruction::NewObject);
                for (key, value) in properties {
                    let key_idx = self.add_constant(Value::String(key.clone()));
                    self.instructions.push(Instruction::LoadConst(key_idx));
                    self.generate_expression(value)?;
                    self.instructions.push(Instruction::SetProperty);
                }
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
            _ => {}
        }
        Ok(())
    }

    fn compile_function(&mut self, name: Option<String>, params: &[String], body: &[Statement]) -> Result<u32> {
        let func_idx = self.functions.len() as u32;
        let parent_locals_snapshot = self.locals.clone();
        let outer_refs = find_outer_refs(body, params, &parent_locals_snapshot);
        let num_captures = outer_refs.len();

        self.functions.push(CompiledFunction {
            name: name.clone(),
            params: params.to_vec(),
            bytecode_index: 0,
            param_count: params.len(),
            closure_var_count: num_captures,
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
        Ok(func_idx)
    }

    fn compile_class_constructor(&mut self, body: &[ClassMember]) -> Result<Option<u32>> {
        for member in body {
            if let ClassMember::Constructor { params, body } = member {
                return Ok(Some(self.compile_function(Some("constructor".to_string()), params, body)?));
            }
        }
        Ok(None)
    }

    fn resolve_local(&self, name: &str) -> Option<u16> {
        for (i, captured_name) in self.captured_var_names.iter().enumerate() {
            if captured_name == name {
                return Some(i as u16);
            }
        }
        let offset = self.captured_var_names.len();
        for (i, local) in self.locals[self.local_start_idx..].iter().enumerate() {
            if local == name {
                return Some((offset + i) as u16);
            }
        }
        None
    }

    fn add_constant(&mut self, value: Value) -> u32 {
        let idx = self.constants.len() as u32;
        self.constants.push(value);
        idx
    }

    fn patch_jump(&mut self, offset: usize, target: usize) {
        if offset >= self.instructions.len() {
            return;
        }
        let target_u32 = target as u32;
        match &mut self.instructions[offset] {
            Instruction::JumpIfNot(addr) => *addr = target_u32,
            Instruction::JumpIf(addr) => *addr = target_u32,
            Instruction::Jump(addr) => *addr = target_u32,
            _ => {}
        }
    }
}

fn find_outer_refs(body: &[Statement], inner_params: &[String], parent_locals: &[String]) -> Vec<(String, u16)> {
    let mut names = Vec::new();
    collect_identifiers_body(body, &mut names);

    let params_set: HashSet<&str> = inner_params.iter().map(|s| s.as_str()).collect();
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for name in &names {
        if params_set.contains(name.as_str()) {
            continue;
        }
        if seen.contains(name.as_str()) {
            continue;
        }
        seen.insert(name.as_str());

        for (i, local) in parent_locals.iter().enumerate() {
            if local == name {
                result.push((name.clone(), i as u16));
                break;
            }
        }
    }

    result
}

fn collect_identifiers_body(body: &[Statement], out: &mut Vec<String>) {
    for stmt in body {
        collect_identifiers_stmt(stmt, out);
    }
}

fn collect_identifiers_stmt(stmt: &Statement, out: &mut Vec<String>) {
    match stmt {
        Statement::Expression(expr) => collect_identifiers_expr(expr, out),
        Statement::VariableDeclaration { declarations, .. } => {
            for decl in declarations {
                if let Some(init) = &decl.init {
                    collect_identifiers_expr(init, out);
                }
            }
        }
        Statement::ReturnStatement(Some(expr)) => collect_identifiers_expr(expr, out),
        Statement::ReturnStatement(None) => {}
        Statement::IfStatement { condition, consequent, alternate } => {
            collect_identifiers_expr(condition, out);
            collect_identifiers_stmt(consequent, out);
            if let Some(alt) = alternate {
                collect_identifiers_stmt(alt, out);
            }
        }
        Statement::WhileStatement { condition, body } => {
            collect_identifiers_expr(condition, out);
            collect_identifiers_stmt(body, out);
        }
        Statement::BlockStatement(stmts) => {
            for s in stmts {
                collect_identifiers_stmt(s, out);
            }
        }
        Statement::ForStatement { init, condition, update, body } => {
            if let Some(for_init) = init {
                match for_init.as_ref() {
                    ForInit::Variable(stmt) => collect_identifiers_stmt(stmt, out),
                    ForInit::Expression(expr) => collect_identifiers_expr(expr, out),
                }
            }
            if let Some(cond) = condition {
                collect_identifiers_expr(cond, out);
            }
            if let Some(upd) = update {
                collect_identifiers_expr(upd, out);
            }
            collect_identifiers_stmt(body, out);
        }
        Statement::ForInStatement { right, body, .. } | Statement::ForOfStatement { right, body, .. } => {
            collect_identifiers_expr(right, out);
            collect_identifiers_stmt(body, out);
        }
        Statement::DoWhileStatement { condition, body } => {
            collect_identifiers_expr(condition, out);
            collect_identifiers_stmt(body, out);
        }
        Statement::SwitchStatement { discriminant, cases } => {
            collect_identifiers_expr(discriminant, out);
            for case in cases {
                if let Some(test) = &case.test {
                    collect_identifiers_expr(test, out);
                }
                collect_identifiers_body(&case.consequent, out);
            }
        }
        Statement::ThrowStatement(expr) => collect_identifiers_expr(expr, out),
        Statement::TryStatement { block, handler, finalizer } => {
            collect_identifiers_body(block, out);
            if let Some(h) = handler {
                collect_identifiers_body(&h.body, out);
            }
            if let Some(f) = finalizer {
                collect_identifiers_body(f, out);
            }
        }
        _ => {}
    }
}

fn collect_identifiers_expr(expr: &Expression, out: &mut Vec<String>) {
    match expr {
        Expression::Identifier(name) => out.push(name.clone()),
        Expression::BinaryOp { left, right, .. } => {
            collect_identifiers_expr(left, out);
            collect_identifiers_expr(right, out);
        }
        Expression::UnaryOp { operand, .. } => {
            collect_identifiers_expr(operand, out);
        }
        Expression::Assignment { target, value, .. } => {
            collect_identifiers_expr(target, out);
            collect_identifiers_expr(value, out);
        }
        Expression::Call { callee, args } => {
            collect_identifiers_expr(callee, out);
            for arg in args {
                collect_identifiers_expr(arg, out);
            }
        }
        Expression::Member { object, property, .. } => {
            collect_identifiers_expr(object, out);
            collect_identifiers_expr(property, out);
        }
        Expression::ConditionalExpression { test, consequent, alternate } => {
            collect_identifiers_expr(test, out);
            collect_identifiers_expr(consequent, out);
            collect_identifiers_expr(alternate, out);
        }
        Expression::UpdateExpression { operand, .. } => {
            collect_identifiers_expr(operand, out);
        }
        Expression::ArrowFunction { body, .. } => {
            match body.as_ref() {
                ArrowFunctionBody::Expression(expr) => collect_identifiers_expr(expr, out),
                ArrowFunctionBody::Block(stmts) => collect_identifiers_body(stmts, out),
            }
        }
        Expression::NewExpression { callee, args } => {
            collect_identifiers_expr(callee, out);
            for arg in args {
                collect_identifiers_expr(arg, out);
            }
        }
        Expression::TemplateLiteral { expressions, .. } => {
            for expr in expressions {
                collect_identifiers_expr(expr, out);
            }
        }
        Expression::AwaitExpression { argument } => {
            collect_identifiers_expr(argument, out);
        }
        _ => {}
    }
}
