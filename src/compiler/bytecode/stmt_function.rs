use crate::compiler::parser::Statement;
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
        if self.scope_depth == 0 {
            self.emit(Instruction::StoreGlobal(name.clone()));
        } else {
            self.locals.push(name.clone());
            let slot = (self.locals.len() - 1 - self.local_start_idx) as u16;
            self.emit(Instruction::StoreLocal(slot));
        }
        Ok(true)
    }
}
