use crate::compiler::parser::{BinaryOperator, BindingPattern, Expression, SpannedNode, Statement};
use crate::compiler::{CompiledFunction, Instruction};
use crate::errors::Result;

use super::{closures, CodeGenerator};

pub(super) fn collect_binding_names(pattern: &BindingPattern, out: &mut Vec<String>) {
    match pattern {
        BindingPattern::Identifier(name) => out.push(name.clone()),
        BindingPattern::Array(elements) => {
            for elem in elements {
                match elem {
                    crate::compiler::parser::ArrayBindingElement::Pattern(pat, _) => {
                        collect_binding_names(pat, out);
                    }
                    crate::compiler::parser::ArrayBindingElement::Rest(pat) => {
                        collect_binding_names(pat, out);
                    }
                    crate::compiler::parser::ArrayBindingElement::Skip => {}
                }
            }
        }
        BindingPattern::Object(elements) => {
            for elem in elements {
                collect_binding_names(&elem.value, out);
            }
        }
    }
}

struct HoistedFunc {
    func_idx: u32,
    slot: u16,
    body: Vec<SpannedNode<Statement>>,
    params: Vec<String>,
    rest_param: Option<String>,
}

impl CodeGenerator {
    pub(super) fn generate_function_statement(&mut self, stmt: &Statement) -> Result<bool> {
        let Statement::FunctionDeclaration {
            name,
            params,
            param_patterns,
            body,
            is_async: _,
            param_types: _,
            return_type: _,
            is_generator,
            defaults,
            rest_param,
        } = stmt
        else {
            return Ok(false);
        };

        let func_idx = self.functions.len() as u32;
        let mut all_params = params.clone();
        if let Some(rp) = rest_param {
            all_params.push(rp.clone());
        }
        let outer_refs = closures::find_outer_refs_with_slots(body, &all_params, |name| {
            self.resolve_local(name)
        });
        let num_captures = outer_refs.len();

        self.functions.push(CompiledFunction {
            name: Some(name.clone()),
            params: params.clone(),
            rest_param: rest_param.clone(),
            bytecode_index: 0,
            param_count: params.len(),
            closure_var_count: num_captures,
            local_count: 0,
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
        let saved_max_locals = self.max_local_count;
        self.captured_var_names = outer_refs.iter().map(|(n, _)| n.clone()).collect();
        self.local_start_idx = self.locals.len();
        self.max_local_count = self.captured_var_names.len();

        for param in params {
            self.locals.push(param.clone());
        }
        if let Some(rp) = rest_param {
            self.locals.push(rp.clone());
        }
        self.note_local_high_water();

        // JavaScript hoisting: pre-register every declaration (function,
        // var/let/const, and class) so sibling functions defined later can
        // resolve them via closure capture regardless of source order.
        self.pre_register_declarations(body);

        self.compile_default_params(params, defaults)?;
        // Emit parameter destructuring for patterned params (e.g. `([a,b]) => ...`).
        for (i, pattern_opt) in param_patterns.iter().enumerate() {
            if let Some(pattern) = pattern_opt {
                let mut names = Vec::new();
                collect_binding_names(pattern, &mut names);
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

        let saved_pending = std::mem::take(&mut self.pending_closure_snapshots);
        let mut deferred_snapshots: Vec<(u16, Vec<u16>)> = Vec::new();
        self.compile_hoisted_functions(body, &mut deferred_snapshots)?;
        // Defer SnapshotClosure until after variable initializers run (and
        // again before returns). Snapshotting here captured `undefined` for
        // every `const`/`let`/`var` that is initialized later in the body.
        self.pending_closure_snapshots = deferred_snapshots;

        // Compile body statements in source order (skip function decls already
        // emitted by compile_hoisted_functions).
        for stmt in body {
            if matches!(&stmt.inner, Statement::FunctionDeclaration { .. }) {
                continue;
            }
            self.record_line_from_span(&stmt.span);
            self.generate_statement(&stmt.inner, false)?;
        }

        self.flush_closure_snapshots();
        self.pending_closure_snapshots = saved_pending;

        self.emit(Instruction::LoadUndefined);
        self.emit(Instruction::Return);

        self.finalize_local_count(func_idx);

        self.scope_depth -= 1;
        self.locals.truncate(prev_locals);
        self.captured_var_names = saved_captured;
        self.local_start_idx = saved_start;
        self.max_local_count = saved_max_locals;

        self.patch_jump(jump_over, self.instructions.len());

        // Always MakeFunction first. When there are outer captures, defer the
        // value snapshot until after the enclosing frame initializes those
        // bindings (pending_closure_snapshots). Immediate MakeClosure here
        // would capture `undefined` for every later `const`/`let`/`var`.
        self.emit(Instruction::MakeFunction(func_idx));
        if self.scope_depth == 0 {
            self.emit(Instruction::StoreGlobal(name.clone()));
            // Top-level: no local slots to snapshot; free vars are globals.
        } else {
            let slot = if let Some(slot) = self.resolve_local(name) {
                self.emit(Instruction::StoreLocal(slot));
                slot
            } else {
                self.locals.push(name.clone());
                let slot = self.last_local_slot();
                self.emit(Instruction::StoreLocal(slot));
                slot
            };
            if num_captures > 0 {
                let capture_slots: Vec<u16> = outer_refs.iter().map(|(_, s)| *s).collect();
                self.pending_closure_snapshots.push((slot, capture_slots));
            }
        }
        Ok(true)
    }

    pub(crate) fn compile_hoisted_functions(
        &mut self,
        body: &[SpannedNode<Statement>],
        deferred_snapshots: &mut Vec<(u16, Vec<u16>)>,
    ) -> Result<()> {
        let mut hoisted: Vec<HoistedFunc> = Vec::new();

        for stmt in body.iter() {
            if let Statement::FunctionDeclaration {
                name: hname,
                params: hparams,
                body: hbody,
                rest_param: hrp,
                ..
            } = &stmt.inner
            {
                let mut all_p = hparams.clone();
                if let Some(rp) = hrp {
                    all_p.push(rp.clone());
                }
                let orefs = closures::find_outer_refs_with_slots(hbody, &all_p, |name| {
                    self.resolve_local(name)
                });

                let slot = self.resolve_local(hname).unwrap_or_else(|| {
                    self.locals.push(hname.clone());
                    self.last_local_slot()
                });

                let func_idx = self.functions.len() as u32;
                hoisted.push(HoistedFunc {
                    func_idx,
                    slot,
                    body: hbody.clone(),
                    params: hparams.clone(),
                    rest_param: hrp.clone(),
                });

                self.functions.push(CompiledFunction {
                    name: Some(hname.clone()),
                    params: hparams.clone(),
                    rest_param: hrp.clone(),
                    bytecode_index: 0,
                    param_count: hparams.len(),
                    closure_var_count: orefs.len(),
                    local_count: 0,
                    is_generator: false,
                    source_line: self.current_source_line,
                    is_arrow: false,
                });

                let jump_over = self.instructions.len();
                self.emit(Instruction::Jump(0));

                let func_start = self.instructions.len();
                self.functions[func_idx as usize].bytecode_index = func_start;

                let saved_scope = self.scope_depth;
                let saved_locals_len = self.locals.len();
                let saved_captured2 = std::mem::take(&mut self.captured_var_names);
                let saved_start2 = self.local_start_idx;

                self.scope_depth += 1;
                self.captured_var_names = orefs.iter().map(|(n, _)| n.clone()).collect();
                self.local_start_idx = self.locals.len();

                for param in hparams {
                    self.locals.push(param.clone());
                }
                if let Some(rp) = hrp {
                    self.locals.push(rp.clone());
                }

                // Nested function decls in this body push onto
                // pending_closure_snapshots; keep them scoped to this body so
                // SnapshotClosure ops are emitted into *this* function.
                let saved_pending_body = std::mem::take(&mut self.pending_closure_snapshots);
                self.generate_body_statements(hbody)?;
                self.flush_closure_snapshots();
                self.pending_closure_snapshots = saved_pending_body;

                self.emit(Instruction::LoadUndefined);
                self.emit(Instruction::Return);

                self.finalize_local_count(func_idx);

                self.scope_depth = saved_scope;
                self.locals.truncate(saved_locals_len);
                self.captured_var_names = saved_captured2;
                self.local_start_idx = saved_start2;

                self.patch_jump(jump_over, self.instructions.len());
            }
        }

        // Phase 2: Create each function object and store it immediately
        for h in &hoisted {
            self.emit(Instruction::MakeFunction(h.func_idx));
            self.emit(Instruction::StoreLocal(h.slot));
        }

        // Phase 3: For functions needing captures, defer SnapshotClosure so it
        // runs after variable initializers (and before returns) via
        // pending_closure_snapshots — not immediately after MakeFunction.
        for h in &hoisted {
            let mut all_p = h.params.clone();
            if let Some(rp) = &h.rest_param {
                all_p.push(rp.clone());
            }
            let orefs = closures::find_outer_refs_with_slots(&h.body, &all_p, |name| {
                self.resolve_local(name)
            });
            if !orefs.is_empty() {
                let capture_slots: Vec<u16> = orefs.iter().map(|(_, s)| *s).collect();
                deferred_snapshots.push((h.slot, capture_slots));
            }
        }

        Ok(())
    }

    fn generate_body_statements(&mut self, body: &[SpannedNode<Statement>]) -> Result<()> {
        self.pre_register_declarations(body);
        // Hoist nested function declarations first (JS semantics). Captures are
        // registered on pending_closure_snapshots and flushed after inits.
        for stmt in body.iter() {
            if let Statement::FunctionDeclaration { .. } = &stmt.inner {
                self.record_line_from_span(&stmt.span);
                self.generate_statement(&stmt.inner, false)?;
            }
        }
        for stmt in body {
            if matches!(&stmt.inner, Statement::FunctionDeclaration { .. }) {
                continue;
            }
            self.record_line_from_span(&stmt.span);
            self.generate_statement(&stmt.inner, false)?;
        }
        Ok(())
    }

    pub(crate) fn compile_default_params(
        &mut self,
        params: &[String],
        defaults: &[Option<Expression>],
    ) -> Result<()> {
        for (i, param_name) in params.iter().enumerate() {
            if let Some(Some(default_expr)) = defaults.get(i) {
                let cond = Expression::BinaryOp {
                    op: BinaryOperator::StrictEq,
                    left: Box::new(Expression::Identifier(param_name.clone())),
                    right: Box::new(Expression::UndefinedLiteral),
                };
                let assign = Statement::Expression(Expression::Assignment {
                    target: Box::new(Expression::Identifier(param_name.clone())),
                    value: Box::new(default_expr.clone()),
                    op: None,
                });
                let if_stmt = Statement::IfStatement {
                    condition: cond,
                    consequent: Box::new(SpannedNode {
                        inner: assign,
                        span: None,
                    }),
                    alternate: None,
                };
                self.generate_statement(&if_stmt, false)?;
            }
        }
        Ok(())
    }
}
