use crate::compiler::parser::{SpannedNode, Statement};
use crate::compiler::{CompiledFunction, Instruction};
use crate::errors::Result;

use super::{closures, CodeGenerator};

impl CodeGenerator {
    pub(super) fn generate_function_statement(&mut self, stmt: &Statement) -> Result<bool> {
        let Statement::FunctionDeclaration {
            name,
            params,
            body,
            is_async: _,
            param_types: _,
            return_type: _,
            is_generator,
            defaults: _,
            rest_param,
        } = stmt
        else {
            return Ok(false);
        };

        let func_idx = self.functions.len() as u32;
        let parent_locals_snapshot = self.locals.clone();
        let mut all_params = params.clone();
        if let Some(rp) = rest_param {
            all_params.push(rp.clone());
        }
        let outer_refs = closures::find_outer_refs(body, &all_params, &parent_locals_snapshot);
        let num_captures = outer_refs.len();

        self.functions.push(CompiledFunction {
            name: Some(name.clone()),
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

        // JavaScript hoisting: pre-register all function names
        for stmt in body.iter() {
            if let Statement::FunctionDeclaration { name, .. } = &stmt.inner {
                self.locals.push(name.clone());
            }
        }
        // Sort function declarations: non-capturing siblings first, then capturing.
        // This ensures that when MakeClosure snapshots a sibling's value, that
        // sibling has already been stored on the stack.
        let sibling_names: Vec<&str> = body
            .iter()
            .filter_map(|stmt| {
                if let Statement::FunctionDeclaration { name, .. } = &stmt.inner {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect();
        let mut func_stmts: Vec<&SpannedNode<Statement>> = body
            .iter()
            .filter(|s| matches!(&s.inner, Statement::FunctionDeclaration { .. }))
            .collect();
        func_stmts.sort_by_key(|stmt| {
            if let Statement::FunctionDeclaration {
                params,
                body: func_body,
                rest_param,
                ..
            } = &stmt.inner
            {
                let mut all_params = params.clone();
                if let Some(rp) = rest_param {
                    all_params.push(rp.clone());
                }
                let mut idents = Vec::new();
                closures::collect_identifiers_body(func_body, &mut idents);
                let captures_sibling = idents.iter().any(|ident| {
                    sibling_names.contains(&ident.as_str()) && !all_params.contains(ident)
                });
                if captures_sibling {
                    1
                } else {
                    0
                }
            } else {
                0
            }
        });
        // Compile function declarations first (function objects exist before other code runs)
        for stmt in func_stmts {
            self.record_line_from_span(&stmt.span);
            self.generate_statement(&stmt.inner, false)?;
        }

        // Then compile non-function statements
        for stmt in body {
            if matches!(&stmt.inner, Statement::FunctionDeclaration { .. }) {
                continue;
            }
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
            self.emit(Instruction::MakeClosure(func_idx, Box::new(capture_slots)));
        } else {
            self.emit(Instruction::MakeFunction(func_idx));
        }
        if self.scope_depth == 0 {
            self.emit(Instruction::StoreGlobal(name.clone()));
        } else if let Some(slot) = self.resolve_local(name) {
            // Name was already hoisted, use the existing slot
            self.emit(Instruction::StoreLocal(slot));
        } else {
            self.locals.push(name.clone());
            let slot = self.last_local_slot();
            self.emit(Instruction::StoreLocal(slot));
        }
        Ok(true)
    }
}
