mod expressions;
mod closures;

use crate::compiler::parser::{AstNode, Statement, Expression, BinaryOperator, UnaryOperator, ForInit, ForInLeft, CompoundAssignmentOp, UpdateOperator, ArrowFunctionBody, ClassMember};
use crate::compiler::{CompiledModule, CompiledFunction, Instruction, ClassInfo, ClassMethodInfo, ClassMethodKind};
use crate::errors::Result;
use crate::objects::Value;

pub fn generate(ast: &AstNode) -> Result<CompiledModule> {
    let mut generator = CodeGenerator::new();
    generator.generate(ast)
}

pub(crate) struct CodeGenerator {
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

    pub(crate) fn generate_statement(&mut self, stmt: &Statement, is_last: bool) -> Result<()> {
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
            Statement::ImportDeclaration { specifiers, source } => {
                if specifiers.is_empty() {
                    self.instructions.push(Instruction::ImportModule(source.clone()));
                } else if specifiers.len() == 1 && specifiers[0].imported.as_deref() == Some("*") {
                    self.instructions.push(Instruction::ImportAll(source.clone(), specifiers[0].local.clone()));
                } else {
                    for spec in specifiers {
                        let imported_name = spec.imported.clone().unwrap_or_else(|| spec.local.clone());
                        if imported_name == "default" {
                            self.instructions.push(Instruction::ImportDefault(source.clone(), spec.local.clone()));
                        } else {
                            self.instructions.push(Instruction::ImportNamed(source.clone(), imported_name, spec.local.clone()));
                        }
                    }
                }
                Ok(())
            }
            Statement::ExportDeclaration { declaration } => {
                match declaration.as_ref() {
                    Statement::VariableDeclaration { declarations, .. } => {
                        let names: Vec<String> = declarations.iter().map(|d| d.id.clone()).collect();
                        self.generate_statement(declaration, false)?;
                        for name in &names {
                            self.instructions.push(Instruction::StoreModuleExport(name.clone()));
                        }
                    }
                    Statement::FunctionDeclaration { name, .. } => {
                        self.generate_statement(declaration, false)?;
                        self.instructions.push(Instruction::StoreModuleExport(name.clone()));
                    }
                    Statement::ClassDeclaration { name, .. } => {
                        self.generate_statement(declaration, false)?;
                        self.instructions.push(Instruction::StoreModuleExport(name.clone()));
                    }
                    _ => {
                        self.generate_statement(declaration, false)?;
                    }
                }
                Ok(())
            }
            Statement::InterfaceDeclaration { .. } => Ok(()),
            Statement::TypeAliasDeclaration { .. } => Ok(()),
            Statement::EnumDeclaration { .. } => Ok(()),
            Statement::ExportDefaultDeclaration { declaration } => {
                match declaration.as_ref() {
                    Statement::FunctionDeclaration { name, .. } => {
                        self.generate_statement(declaration, false)?;
                        self.instructions.push(Instruction::StoreModuleExport(name.clone()));
                        self.instructions.push(Instruction::LoadGlobal(name.clone()));
                        self.instructions.push(Instruction::ExportDefault);
                    }
                    Statement::ClassDeclaration { name, .. } => {
                        self.generate_statement(declaration, false)?;
                        self.instructions.push(Instruction::StoreModuleExport(name.clone()));
                        self.instructions.push(Instruction::LoadGlobal(name.clone()));
                        self.instructions.push(Instruction::ExportDefault);
                    }
                    _ => {
                        self.generate_statement(declaration, false)?;
                        self.instructions.push(Instruction::ExportDefault);
                    }
                }
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
            Statement::FunctionDeclaration { name, params, body, is_async: _, param_types: _, return_type: _ } => {
                let func_idx = self.functions.len() as u32;
                let parent_locals_snapshot = self.locals.clone();
                let outer_refs = closures::find_outer_refs(body, params, &parent_locals_snapshot);
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

    pub(crate) fn compile_class_constructor(&mut self, body: &[ClassMember]) -> Result<Option<u32>> {
        for member in body {
            if let ClassMember::Constructor { params, body } = member {
                return Ok(Some(self.compile_function(Some("constructor".to_string()), params, body)?));
            }
        }
        Ok(None)
    }

    pub(crate) fn resolve_local(&self, name: &str) -> Option<u16> {
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

    pub(crate) fn add_constant(&mut self, value: Value) -> u32 {
        let idx = self.constants.len() as u32;
        self.constants.push(value);
        idx
    }

    pub(crate) fn patch_jump(&mut self, offset: usize, target: usize) {
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
