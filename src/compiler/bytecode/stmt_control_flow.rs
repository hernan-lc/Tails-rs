use super::CodeGenerator;
use crate::compiler::parser::{BindingPattern, Expression, ForInLeft, ForInit, Statement};
use crate::compiler::Instruction;
use crate::errors::Result;
use crate::objects::Value;

impl CodeGenerator {
    pub(crate) fn generate_control_flow_statement(
        &mut self,
        stmt: &Statement,
        is_last: bool,
    ) -> Result<bool> {
        if self.generate_control_flow_inner(stmt, is_last)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn generate_control_flow_inner(&mut self, stmt: &Statement, _is_last: bool) -> Result<bool> {
        match stmt {
            Statement::IfStatement {
                condition,
                consequent,
                alternate,
            } => {
                self.generate_expression(condition)?;
                let jump_if_not = self.instructions.len();
                self.emit(Instruction::JumpIfNot(0));
                self.record_line_from_span(&consequent.span);
                self.generate_statement_in_branch(&consequent.inner)?;
                if let Some(alt) = alternate {
                    let jump_to_end = self.instructions.len();
                    self.emit(Instruction::Jump(0));
                    self.patch_jump(jump_if_not, self.instructions.len());
                    self.record_line_from_span(&alt.span);
                    self.generate_statement_in_branch(&alt.inner)?;
                    self.patch_jump(jump_to_end, self.instructions.len());
                } else {
                    self.patch_jump(jump_if_not, self.instructions.len());
                }
                Ok(true)
            }
            Statement::WhileStatement { condition, body } => {
                let loop_start = self.instructions.len() as u32;
                let break_start = self.break_targets.len();
                self.break_targets.push(usize::MAX);
                self.generate_expression(condition)?;
                let jump_if_not = self.instructions.len();
                self.emit(Instruction::JumpIfNot(0));
                self.continue_targets.push(loop_start as usize);
                self.record_line_from_span(&body.span);
                self.generate_statement(&body.inner, false)?;
                self.emit(Instruction::Jump(loop_start));
                let loop_end = self.instructions.len();
                self.patch_jump(jump_if_not, loop_end);
                while self.break_targets.len() > break_start {
                    let idx = self.break_targets.pop().unwrap();
                    self.patch_jump(idx, loop_end);
                }
                self.continue_targets.pop();
                Ok(true)
            }
            Statement::ForStatement {
                init,
                condition,
                update,
                body,
            } => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();

                if let Some(for_init) = init {
                    match for_init.as_ref() {
                        ForInit::Variable(stmt) => {
                            self.record_line_from_span(&stmt.span);
                            self.generate_statement(&stmt.inner, false)?;
                        }
                        ForInit::Expression(expr) => {
                            self.generate_expression(expr)?;
                            self.emit(Instruction::Pop);
                        }
                    }
                }

                // Jump to condition check first
                let jump_to_check = self.instructions.len();
                self.emit(Instruction::Jump(0));

                // Body start (continue jumps here via patching)
                let body_start = self.instructions.len() as u32;

                // Push placeholder for continue target
                self.continue_targets.push(usize::MAX);
                let cont_start = self.continue_patches.len();
                let break_start = self.break_targets.len();
                self.break_targets.push(usize::MAX);
                self.record_line_from_span(&body.span);
                self.generate_statement(&body.inner, false)?;

                // Update expression (patch continues to here)
                while self.continue_patches.len() > cont_start {
                    let idx = self.continue_patches.pop().unwrap();
                    self.patch_jump(idx, self.instructions.len());
                }
                self.continue_targets.pop();

                if let Some(upd) = update {
                    let is_assignment = matches!(upd, Expression::Assignment { .. });
                    self.generate_expression(upd)?;
                    if !is_assignment {
                        self.emit(Instruction::Pop);
                    }
                }

                // Condition check (patched from jump_to_check)
                self.patch_jump(jump_to_check, self.instructions.len());
                if let Some(cond) = condition {
                    self.generate_expression(cond)?;
                    let j = self.instructions.len();
                    self.emit(Instruction::JumpIfNot(0));
                    // If condition true, jump back to body
                    self.emit(Instruction::Jump(body_start));
                    let loop_end = self.instructions.len();
                    self.patch_jump(j, loop_end);
                    while self.break_targets.len() > break_start {
                        let idx = self.break_targets.pop().unwrap();
                        self.patch_jump(idx, loop_end);
                    }
                } else {
                    // No condition - infinite loop
                    self.emit(Instruction::Jump(body_start));
                    let loop_end = self.instructions.len();
                    while self.break_targets.len() > break_start {
                        let idx = self.break_targets.pop().unwrap();
                        self.patch_jump(idx, loop_end);
                    }
                }

                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                    self.emit(Instruction::Pop);
                }
                self.scope_depth -= 1;
                Ok(true)
            }
            Statement::ForInStatement { left, right, body } => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();

                let var_name = match left {
                    ForInLeft::Identifier(id) => id.clone(),
                    ForInLeft::VariableDeclaration {
                        id: BindingPattern::Identifier(name),
                        ..
                    } => name.clone(),
                    ForInLeft::VariableDeclaration {
                        id: BindingPattern::Array(_),
                        ..
                    }
                    | ForInLeft::VariableDeclaration {
                        id: BindingPattern::Object(_),
                        ..
                    }
                    | ForInLeft::Pattern(_) => String::new(),
                };

                let is_destructuring = matches!(
                    left,
                    ForInLeft::VariableDeclaration {
                        id: BindingPattern::Array(_) | BindingPattern::Object(_),
                        ..
                    } | ForInLeft::Pattern(_)
                );

                if !is_destructuring && !var_name.is_empty() {
                    self.locals.push(var_name.clone());
                }
                let var_slot = if !var_name.is_empty() {
                    self.last_local_slot()
                } else {
                    0
                };

                // Evaluate the object and get its keys
                self.generate_expression(right)?;
                self.emit(Instruction::GetKeys);
                let keys_slot = self.current_local_slot();
                self.locals.push("__keys".to_string());
                self.emit(Instruction::StoreLocal(keys_slot));

                // Initialize index
                let idx_slot = self.current_local_slot();
                self.locals.push("__idx".to_string());
                let zero_idx = self.add_constant(Value::Float(0.0));
                self.emit(Instruction::LoadConst(zero_idx));
                self.emit(Instruction::StoreLocal(idx_slot));

                // Get keys.length
                let len_slot = self.current_local_slot();
                self.locals.push("__len".to_string());
                self.emit(Instruction::LoadLocal(keys_slot));
                let len_key = self.add_constant(Value::String("length".to_string()));
                self.emit(Instruction::LoadConst(len_key));
                self.emit(Instruction::GetProperty);
                self.emit(Instruction::StoreLocal(len_slot));

                let loop_start = self.instructions.len() as u32;

                // Check if idx < len
                self.emit(Instruction::LoadLocal(idx_slot));
                self.emit(Instruction::LoadLocal(len_slot));
                self.emit(Instruction::Less);
                let jump_if_done = self.instructions.len();
                self.emit(Instruction::JumpIfNot(0));

                // Get keys[idx] and store in loop variable
                self.emit(Instruction::LoadLocal(keys_slot));
                self.emit(Instruction::LoadLocal(idx_slot));
                self.emit(Instruction::GetProperty);
                if is_destructuring {
                    let pattern = match left {
                        ForInLeft::VariableDeclaration { id, .. } | ForInLeft::Pattern(id) => id,
                        _ => unreachable!(),
                    };
                    self.generate_destructuring_pattern(pattern)?;
                } else {
                    self.emit(Instruction::StoreLocal(var_slot));
                }

                self.record_line_from_span(&body.span);
                self.generate_statement(&body.inner, false)?;

                // Increment index
                self.emit(Instruction::LoadLocal(idx_slot));
                let one_idx = self.add_constant(Value::Float(1.0));
                self.emit(Instruction::LoadConst(one_idx));
                self.emit(Instruction::Add);
                self.emit(Instruction::StoreLocal(idx_slot));

                self.emit(Instruction::Jump(loop_start));

                self.patch_jump(jump_if_done, self.instructions.len());

                self.locals.pop();
                self.locals.pop();
                self.locals.pop();

                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                    self.emit(Instruction::Pop);
                }
                self.scope_depth -= 1;
                Ok(true)
            }
            Statement::ForOfStatement {
                left,
                right,
                body,
                is_async,
            } => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();

                let var_name = match left {
                    ForInLeft::Identifier(id) => id.clone(),
                    ForInLeft::VariableDeclaration {
                        id: BindingPattern::Identifier(name),
                        ..
                    } => name.clone(),
                    ForInLeft::VariableDeclaration {
                        id: BindingPattern::Array(_),
                        ..
                    }
                    | ForInLeft::VariableDeclaration {
                        id: BindingPattern::Object(_),
                        ..
                    }
                    | ForInLeft::Pattern(_) => String::new(),
                };

                let is_destructuring = matches!(
                    left,
                    ForInLeft::VariableDeclaration {
                        id: BindingPattern::Array(_) | BindingPattern::Object(_),
                        ..
                    } | ForInLeft::Pattern(_)
                );

                if !is_destructuring && !var_name.is_empty() {
                    self.emit(Instruction::LoadUndefined);
                    self.locals.push(var_name.clone());
                }
                let var_slot = if !var_name.is_empty() {
                    self.last_local_slot()
                } else {
                    0
                };

                // Evaluate the iterable and get an iterator
                self.generate_expression(right)?;
                if *is_async {
                    self.emit(Instruction::GetAsyncIterator);
                } else {
                    self.emit(Instruction::GetIterator);
                }

                // Store iterator in a local
                let iter_slot = self.current_local_slot();
                self.locals.push("__iter".to_string());
                self.emit(Instruction::StoreLocal(iter_slot));

                let loop_start = self.instructions.len() as u32;

                // Load iterator for IteratorNext
                self.emit(Instruction::LoadLocal(iter_slot));
                // IteratorNext: calls next(), if done jumps to target, else pushes value
                let iter_next_pos = self.instructions.len();
                if *is_async {
                    self.emit(Instruction::AsyncIteratorNext(0)); // placeholder
                } else {
                    self.emit(Instruction::IteratorNext(0)); // placeholder
                }

                // Store the yielded value into the loop variable
                if is_destructuring {
                    let pattern = match left {
                        ForInLeft::VariableDeclaration { id, .. } | ForInLeft::Pattern(id) => id,
                        _ => unreachable!(),
                    };
                    self.generate_destructuring_pattern(pattern)?;
                } else {
                    self.emit(Instruction::StoreLocal(var_slot));
                }

                // Execute loop body
                self.record_line_from_span(&body.span);
                self.generate_statement(&body.inner, false)?;

                // Jump back to loop start
                self.emit(Instruction::Jump(loop_start));

                // Patch IteratorNext jump target (when done)
                let loop_end = self.instructions.len() as u32;
                if *is_async {
                    if let Instruction::AsyncIteratorNext(ref mut target) =
                        self.instructions[iter_next_pos]
                    {
                        *target = loop_end;
                    }
                } else {
                    if let Instruction::IteratorNext(ref mut target) =
                        self.instructions[iter_next_pos]
                    {
                        *target = loop_end;
                    }
                }

                self.locals.pop(); // __iter

                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                    self.emit(Instruction::Pop);
                }
                self.scope_depth -= 1;
                Ok(true)
            }
            Statement::DoWhileStatement { condition, body } => {
                let loop_start = self.instructions.len() as u32;

                let break_start = self.break_targets.len();
                self.break_targets.push(usize::MAX);
                self.record_line_from_span(&body.span);
                self.generate_statement(&body.inner, false)?;

                self.generate_expression(condition)?;

                let jump_if_not = self.instructions.len();
                self.emit(Instruction::JumpIfNot(0));

                self.emit(Instruction::Jump(loop_start));

                let loop_end = self.instructions.len();
                self.patch_jump(jump_if_not, loop_end);
                while self.break_targets.len() > break_start {
                    let idx = self.break_targets.pop().unwrap();
                    self.patch_jump(idx, loop_end);
                }

                Ok(true)
            }
            Statement::SwitchStatement {
                discriminant,
                cases,
            } => {
                self.generate_expression(discriminant)?;

                self.locals.push("__switch_val".to_string());
                let disc_slot = self.last_local_slot();
                self.emit(Instruction::StoreLocal(disc_slot));

                let mut body_jumps: Vec<usize> = Vec::new();
                let mut default_jump: Option<usize> = None;

                for case in cases {
                    if let Some(test) = &case.test {
                        self.emit(Instruction::LoadLocal(disc_slot));
                        self.generate_expression(test)?;
                        self.emit(Instruction::StrictEq);
                        let j = self.instructions.len();
                        self.emit(Instruction::JumpIf(0));
                        body_jumps.push(j);
                    } else {
                        default_jump = Some(self.instructions.len());
                        self.emit(Instruction::Jump(0));
                    }
                }

                let end_jump = self.instructions.len();
                self.emit(Instruction::Jump(0));

                let break_start = self.break_targets.len();
                self.break_targets.push(usize::MAX);
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
                        self.record_line_from_span(&s.span);
                        self.generate_statement(&s.inner, false)?;
                    }
                }

                let loop_end = self.instructions.len();
                while self.break_targets.len() > break_start {
                    let idx = self.break_targets.pop().unwrap();
                    self.patch_jump(idx, loop_end);
                }

                self.patch_jump(end_jump, loop_end);

                self.locals.pop();
                self.emit(Instruction::Pop);

                Ok(true)
            }
            Statement::BreakStatement => {
                if !self.break_targets.is_empty() {
                    self.emit(Instruction::Jump(0));
                    self.break_targets.push(self.instructions.len() - 1);
                } else {
                    self.emit(Instruction::LoadUndefined);
                    self.emit(Instruction::Return);
                }
                Ok(true)
            }
            Statement::ContinueStatement => {
                if let Some(target) = self.continue_targets.last().copied() {
                    if target == usize::MAX {
                        // placeholder - push Jump(0) and record for patching
                        self.emit(Instruction::Jump(0));
                        self.continue_patches.push(self.instructions.len() - 1);
                    } else {
                        self.emit(Instruction::Jump(target as u32));
                    }
                } else {
                    self.emit(Instruction::LoadUndefined);
                    self.emit(Instruction::Return);
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
