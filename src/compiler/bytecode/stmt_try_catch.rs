use super::CodeGenerator;
use crate::compiler::parser::Statement;
use crate::compiler::Instruction;
use crate::errors::Result;

impl CodeGenerator {
    pub(super) fn generate_try_catch_statement(&mut self, stmt: &Statement) -> Result<bool> {
        match stmt {
            Statement::TryStatement {
                block,
                handler,
                finalizer,
            } => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();

                let has_catch = handler.is_some();
                let has_finally = finalizer.is_some();

                self.emit(Instruction::TryJump(0, 0));
                let try_jump_idx = self.instructions.len() - 1;

                for stmt in block {
                    self.record_line_from_span(&stmt.span);
                    self.generate_statement(&stmt.inner, false)?;
                }

                self.emit(Instruction::PopTryHandler);

                let jump_past_catch = if has_catch {
                    let j = self.instructions.len();
                    self.emit(Instruction::Jump(0));
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
                    let slot = self.last_local_slot();
                    self.emit(Instruction::LoadException);
                    self.emit(Instruction::StoreLocal(slot));
                    let last_idx = catch_clause.body.len().saturating_sub(1);
                    for (i, stmt) in catch_clause.body.iter().enumerate() {
                        self.record_line_from_span(&stmt.span);
                        self.generate_statement(&stmt.inner, i == last_idx && !has_finally)?;
                    }
                }

                let finally_pc = self.instructions.len() as u32;

                if let Some(j) = jump_past_catch {
                    self.patch_jump(j, finally_pc as usize);
                }

                if let Instruction::TryJump(ref mut c, ref mut f) = self.instructions[try_jump_idx]
                {
                    *c = catch_pc;
                    *f = finally_pc;
                }

                if has_finally {
                    for stmt in finalizer.as_ref().unwrap() {
                        self.record_line_from_span(&stmt.span);
                        self.generate_statement(&stmt.inner, false)?;
                    }
                    if !has_catch {
                        self.emit(Instruction::ReThrowIfPending);
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
            Statement::ThrowStatement(argument) => {
                self.generate_expression(argument)?;
                self.emit(Instruction::Throw);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
