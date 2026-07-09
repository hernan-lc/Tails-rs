mod closures;
mod expressions;
mod stmt_class;
mod stmt_control_flow;
mod stmt_function;
mod stmt_module;
mod stmt_try_catch;

use crate::compiler::parser::{
    ArrayBindingElement, ArrowFunctionBody, AstNode, BinaryOperator, BindingPattern, ClassMember,
    CompoundAssignmentOp, Expression, SpannedNode, Statement, UnaryOperator, UpdateOperator,
};
use crate::compiler::{
    ClassInfo, ClassMethodInfo, ClassMethodKind, CompiledFunction, CompiledModule, Instruction,
};
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
    continue_patches: Vec<usize>,
    class_infos: Vec<ClassInfo>,
    source_lines: Vec<Option<usize>>,
    source_cols: Vec<Option<usize>>,
    current_source_line: Option<usize>,
    current_source_col: Option<usize>,
    /// Hoisted function declarations that need SnapshotClosure after outer
    /// bindings are initialized. Re-emitted after variable declarations and
    /// before returns so captures see real values (not undefined).
    pending_closure_snapshots: Vec<(u16, Vec<u16>)>,
    /// ES name inference: `var app = function(){}` gives the function name "app".
    /// Set around expression generation when the binding name is known.
    pub(crate) inferred_function_name: Option<String>,
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
            continue_patches: Vec::new(),
            class_infos: Vec::new(),
            source_lines: Vec::new(),
            source_cols: Vec::new(),
            current_source_line: None,
            current_source_col: None,
            pending_closure_snapshots: Vec::new(),
            inferred_function_name: None,
        }
    }

    fn emit(&mut self, instr: Instruction) {
        self.instructions.push(instr);
        self.source_lines.push(self.current_source_line);
        self.source_cols.push(self.current_source_col);
    }

    /// Refresh hoisted closures from current local slot values.
    pub(crate) fn flush_closure_snapshots(&mut self) {
        for (local_slot, capture_slots) in &self.pending_closure_snapshots {
            self.instructions.push(Instruction::SnapshotClosure(
                *local_slot,
                Box::new(capture_slots.clone()),
            ));
            self.source_lines.push(self.current_source_line);
            self.source_cols.push(self.current_source_col);
        }
    }

    fn peephole_optimize(&mut self) {
        let mut optimized = Vec::with_capacity(self.instructions.len());
        let mut keep = Vec::with_capacity(self.instructions.len());
        // old_to_new[i] = Some(new_index) if instruction i was kept, None if removed.
        let mut old_to_new: Vec<Option<usize>> = vec![None; self.instructions.len()];
        let mut i = 0;
        while i < self.instructions.len() {
            // Pattern 1: LoadLocal(x) + LoadConst(n) + Add + StoreLocal(x) → IncLocal(x, n)
            if i + 3 < self.instructions.len() {
                if let (
                    Instruction::LoadLocal(x),
                    Instruction::LoadConst(c),
                    Instruction::Add,
                    Instruction::StoreLocal(y),
                ) = (
                    &self.instructions[i],
                    &self.instructions[i + 1],
                    &self.instructions[i + 2],
                    &self.instructions[i + 3],
                ) {
                    if x == y {
                        if let Value::Integer(n) = self.constants[*c as usize] {
                            old_to_new[i] = Some(optimized.len());
                            optimized.push(Instruction::IncLocal(*x, n));
                            keep.push(i);
                            i += 4;
                            continue;
                        }
                    }
                }
            }

            // Pattern 2: LoadLocal(x) + LoadLocal(y) + Add + StoreLocal(x) → AddLocal(x, y)
            if i + 3 < self.instructions.len() {
                if let (
                    Instruction::LoadLocal(x),
                    Instruction::LoadLocal(y),
                    Instruction::Add,
                    Instruction::StoreLocal(z),
                ) = (
                    &self.instructions[i],
                    &self.instructions[i + 1],
                    &self.instructions[i + 2],
                    &self.instructions[i + 3],
                ) {
                    if x == z && x != y {
                        old_to_new[i] = Some(optimized.len());
                        optimized.push(Instruction::AddLocal(*x, *y));
                        keep.push(i);
                        i += 4;
                        continue;
                    }
                }
            }

            // Pattern 2b: LoadGlobal(name) + LoadLocal(y) + Add + StoreGlobal(name)
            //              → AddGlobal(name, y)
            if i + 3 < self.instructions.len() {
                if let (
                    Instruction::LoadGlobal(name),
                    Instruction::LoadLocal(y),
                    Instruction::Add,
                    Instruction::StoreGlobal(name2),
                ) = (
                    &self.instructions[i],
                    &self.instructions[i + 1],
                    &self.instructions[i + 2],
                    &self.instructions[i + 3],
                ) {
                    if name == name2 {
                        old_to_new[i] = Some(optimized.len());
                        optimized.push(Instruction::AddGlobal(name.clone(), *y));
                        keep.push(i);
                        i += 4;
                        continue;
                    }
                }
            }

            // Pattern 3: Pop after side-effect-free instruction → remove both.
            // NOTE: `LoadGlobal` is intentionally NOT included — resolving an
            // undeclared free variable throws ReferenceError (used by try/catch
            // feature-detection patterns like `try { localStorage } catch {}`).
            if i + 1 < self.instructions.len() {
                if let Instruction::Pop = &self.instructions[i + 1] {
                    match &self.instructions[i] {
                        Instruction::LoadConst(_)
                        | Instruction::LoadLocal(_)
                        | Instruction::LoadNull
                        | Instruction::LoadUndefined
                        | Instruction::LoadTrue
                        | Instruction::LoadFalse
                        | Instruction::LoadGlobalOrUndefined(_) => {
                            i += 2;
                            continue;
                        }
                        _ => {}
                    }
                }
            }

            old_to_new[i] = Some(optimized.len());
            optimized.push(self.instructions[i].clone());
            keep.push(i);
            i += 1;
        }

        // Remap all jump targets to account for removed instructions.
        for instr in &mut optimized {
            let target = match instr {
                Instruction::Jump(t)
                | Instruction::JumpIf(t)
                | Instruction::JumpIfNot(t)
                | Instruction::JumpIfUndefined(t)
                | Instruction::JumpIfNotUndefined(t) => Some(t),
                _ => None,
            };
            if let Some(t) = target {
                let old = *t as usize;
                if old < old_to_new.len() {
                    if let Some(Some(new)) = old_to_new.get(old) {
                        *t = *new as u32;
                    }
                }
            }
        }

        // Remap function bytecode_index values to account for removed instructions.
        for func in &mut self.functions {
            let old_idx = func.bytecode_index;
            if old_idx < old_to_new.len() {
                if let Some(Some(new_idx)) = old_to_new.get(old_idx) {
                    func.bytecode_index = *new_idx;
                }
            }
        }

        self.instructions = optimized;
        self.source_lines = keep.iter().map(|&i| self.source_lines[i]).collect();
        self.source_cols = keep.iter().map(|&i| self.source_cols[i]).collect();
    }

    fn record_line_from_span(&mut self, span: &Option<crate::errors::Span>) {
        if let Some(s) = span {
            if s.line > 0 {
                self.current_source_line = Some(s.line);
                self.current_source_col = if s.col > 0 { Some(s.col) } else { None };
            } else {
                self.current_source_line = None;
                self.current_source_col = None;
            }
        } else {
            self.current_source_line = None;
            self.current_source_col = None;
        }
    }

    fn generate(&mut self, ast: &AstNode) -> Result<CompiledModule> {
        match ast {
            AstNode::Program(statements) => {
                // JavaScript hoisting: emit all top-level function declarations
                // first so they are defined (as globals) before any other
                // statement runs, regardless of source order.
                for stmt in statements {
                    if matches!(&stmt.inner, Statement::FunctionDeclaration { .. }) {
                        self.record_line_from_span(&stmt.span);
                        self.generate_statement(&stmt.inner, false)?;
                    }
                }
                for (i, stmt) in statements.iter().enumerate() {
                    if matches!(&stmt.inner, Statement::FunctionDeclaration { .. }) {
                        continue;
                    }
                    let is_last = statements[i + 1..]
                        .iter()
                        .all(|s| matches!(&s.inner, Statement::FunctionDeclaration { .. }));
                    self.record_line_from_span(&stmt.span);
                    self.generate_statement(&stmt.inner, is_last)?;
                }
                if statements.is_empty() {
                    self.emit(Instruction::LoadUndefined);
                }
            }
            _ => {
                return Err(crate::errors::Error::InternalError(
                    "Invalid AST node".into(),
                ))
            }
        }

        self.peephole_optimize();

        Ok(CompiledModule {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
            functions: self.functions.clone(),
            class_infos: self.class_infos.clone(),
            source_lines: self.source_lines.clone(),
            source_cols: self.source_cols.clone(),
        })
    }

    pub(crate) fn generate_statement(&mut self, stmt: &Statement, is_last: bool) -> Result<()> {
        if self.generate_control_flow_statement(stmt, is_last)? {
            return Ok(());
        }
        if self.generate_try_catch_statement(stmt)? {
            return Ok(());
        }
        if self.generate_class_statement(stmt)? {
            return Ok(());
        }
        if self.generate_module_statement(stmt)? {
            return Ok(());
        }
        if self.generate_function_statement(stmt)? {
            return Ok(());
        }
        match stmt {
            Statement::EmptyStatement => Ok(()),
            Statement::Expression(expr) => {
                self.generate_expression(expr)?;
                if !is_last {
                    self.emit(Instruction::Pop);
                }
                Ok(())
            }
            Statement::VariableDeclaration {
                kind: _,
                declarations,
            } => {
                for decl in declarations {
                    if let Some(init) = &decl.init {
                        // ES name inference: `var app = function() {}` → name "app"
                        if let BindingPattern::Identifier(id) = &decl.id {
                            if matches!(
                                init,
                                Expression::FunctionExpression { name: None, .. }
                                    | Expression::ArrowFunction { .. }
                            ) {
                                self.inferred_function_name = Some(id.clone());
                            }
                        }
                        self.generate_expression(init)?;
                        self.inferred_function_name = None;
                        self.generate_destructuring_pattern(&decl.id)?;
                    } else {
                        match &decl.id {
                            BindingPattern::Identifier(id) => {
                                self.emit(Instruction::LoadUndefined);
                                if self.scope_depth == 0 {
                                    self.emit(Instruction::StoreGlobal(id.clone()));
                                } else {
                                    if !self.locals.iter().any(|l| l == id) {
                                        self.locals.push(id.clone());
                                    }
                                    let slot = self.resolve_local(id).unwrap_or_else(|| self.last_local_slot());
                                    self.emit(Instruction::StoreLocal(slot));
                                }
                            }
                            _ => {
                                self.emit(Instruction::LoadUndefined);
                                self.generate_destructuring_pattern(&decl.id)?;
                            }
                        }
                    }
                }
                // Hoisted nested functions must re-capture after bindings init.
                self.flush_closure_snapshots();
                Ok(())
            }
            Statement::ReturnStatement(value) => {
                // Capture current local values before leaving the frame.
                self.flush_closure_snapshots();
                if let Some(expr) = value {
                    self.generate_expression(expr)?;
                } else {
                    self.emit(Instruction::LoadUndefined);
                }
                self.emit(Instruction::Return);
                Ok(())
            }
            Statement::YieldStatement(value) => {
                if let Some(expr) = value {
                    self.generate_expression(expr)?;
                } else {
                    self.emit(Instruction::LoadUndefined);
                }
                self.emit(Instruction::Yield);
                Ok(())
            }
            Statement::BlockStatement(stmts) => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();
                self.emit(Instruction::BlockEnter);
                for stmt in stmts.iter() {
                    if let Statement::FunctionDeclaration { name, .. } = &stmt.inner {
                        self.locals.push(name.clone());
                    }
                }
                let saved_pending = std::mem::take(&mut self.pending_closure_snapshots);
                let mut deferred_snapshots: Vec<(u16, Vec<u16>)> = Vec::new();
                self.compile_hoisted_functions(stmts, &mut deferred_snapshots)?;
                self.pending_closure_snapshots = deferred_snapshots;
                for (i, stmt) in stmts.iter().enumerate() {
                    if matches!(&stmt.inner, Statement::FunctionDeclaration { .. }) {
                        continue;
                    }
                    let is_last = stmts[i + 1..].iter().all(|s| matches!(&s.inner, Statement::FunctionDeclaration { .. }));
                    self.record_line_from_span(&stmt.span);
                    self.generate_statement(&stmt.inner, is_last)?;
                }
                self.flush_closure_snapshots();
                self.pending_closure_snapshots = saved_pending;
                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                }
                self.emit(Instruction::BlockExit);
                self.scope_depth -= 1;
                Ok(())
            }
            _ => unreachable!("handled by generate_control_flow_statement, generate_try_catch_statement, generate_class_statement, generate_module_statement, or generate_function_statement"),
        }
    }

    // Pre-register every declaration

    // Pre-register every declaration in a function/block body so that lexical
    // bindings (function, var/let/const, and class) are visible to *sibling*
    // functions defined later in the same scope. In JavaScript, a class (or
    // const) declared in a function body is in scope for any nested function
    // declared afterwards; without this, a hoisted sibling function that
    // references the class resolves it to `undefined` at capture time.
    pub(crate) fn pre_register_declarations(&mut self, body: &[SpannedNode<Statement>]) {
        for stmt in body {
            match &stmt.inner {
                Statement::FunctionDeclaration { name, .. } => {
                    if !self.locals.iter().any(|l| l == name) {
                        self.locals.push(name.clone());
                    }
                }
                Statement::VariableDeclaration { declarations, .. } => {
                    for decl in declarations {
                        let mut names = Vec::new();
                        stmt_function::collect_binding_names(&decl.id, &mut names);
                        for name in names {
                            if !self.locals.iter().any(|l| l == &name) {
                                self.locals.push(name);
                            }
                        }
                    }
                }
                Statement::ClassDeclaration { name, .. }
                    if !self.locals.iter().any(|l| l == name) =>
                {
                    self.locals.push(name.clone());
                }
                _ => {}
            }
        }
    }

    fn generate_statement_in_branch(&mut self, stmt: &Statement, leave_value: bool) -> Result<()> {
        match stmt {
            Statement::BlockStatement(stmts) => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();
                self.emit(Instruction::BlockEnter);
                self.pre_register_declarations(stmts);
                let saved_pending = std::mem::take(&mut self.pending_closure_snapshots);
                let mut deferred_snapshots: Vec<(u16, Vec<u16>)> = Vec::new();
                self.compile_hoisted_functions(stmts, &mut deferred_snapshots)?;
                self.pending_closure_snapshots = deferred_snapshots;
                // Source order for all non-function statements (including var
                // initializers — only bindings are hoisted, not initializers).
                // Snapshots flush after var decls / before returns via
                // pending_closure_snapshots (not before initializers).
                let body: Vec<_> = stmts
                    .iter()
                    .filter(|s| !matches!(&s.inner, Statement::FunctionDeclaration { .. }))
                    .collect();
                let last_idx = body.len().saturating_sub(1);
                for (i, stmt) in body.iter().enumerate() {
                    self.record_line_from_span(&stmt.span);
                    let is_last = leave_value && i == last_idx;
                    self.generate_statement(&stmt.inner, is_last)?;
                }
                self.flush_closure_snapshots();
                self.pending_closure_snapshots = saved_pending;
                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                }
                self.emit(Instruction::BlockExit);
                self.scope_depth -= 1;
                Ok(())
            }
            _ => self.generate_statement(stmt, leave_value),
        }
    }

    pub(crate) fn generate_destructuring_pattern(
        &mut self,
        pattern: &BindingPattern,
    ) -> Result<()> {
        match pattern {
            BindingPattern::Identifier(id) => {
                if self.scope_depth == 0 {
                    self.emit(Instruction::StoreGlobal(id.clone()));
                } else {
                    if !self.locals.iter().any(|l| l == id) {
                        self.locals.push(id.clone());
                    }
                    let slot = self
                        .resolve_local(id)
                        .unwrap_or_else(|| self.last_local_slot());
                    self.emit(Instruction::StoreLocal(slot));
                }
            }
            BindingPattern::Array(elements) => {
                for (i, element) in elements.iter().enumerate() {
                    match element {
                        ArrayBindingElement::Pattern(pat, default) => {
                            self.emit(Instruction::Dup);
                            let idx = self.add_constant(Value::Integer(i as i64));
                            self.emit(Instruction::LoadConst(idx));
                            self.emit(Instruction::GetProperty);
                            if let Some(default_expr) = default.as_ref() {
                                let skip_default = self.instructions.len();
                                self.emit(Instruction::JumpIfNotUndefined(0));
                                self.emit(Instruction::Pop);
                                self.generate_expression(default_expr)?;
                                self.patch_jump(skip_default, self.instructions.len());
                            }
                            self.generate_destructuring_pattern(pat)?;
                        }
                        ArrayBindingElement::Rest(pat) => {
                            self.emit(Instruction::Dup);
                            let idx = self.add_constant(Value::from_string("slice".to_string()));
                            self.emit(Instruction::LoadConst(idx));
                            let start_idx = self.add_constant(Value::Integer(i as i64));
                            self.emit(Instruction::LoadConst(start_idx));
                            self.emit(Instruction::CallMethod(1));
                            self.generate_destructuring_pattern(pat)?;
                        }
                        ArrayBindingElement::Skip => {
                            // Skip element
                        }
                    }
                }
                self.emit(Instruction::Pop);
            }
            BindingPattern::Object(elements) => {
                let excluded_keys: Vec<String> = elements
                    .iter()
                    .filter(|e| !e.is_rest)
                    .map(|e| e.key.clone())
                    .collect();
                for element in elements {
                    if element.is_rest {
                        self.emit(Instruction::Dup);
                        self.emit(Instruction::ObjectRest(Box::new(excluded_keys.clone())));
                        self.generate_destructuring_pattern(&element.value)?;
                    } else {
                        self.emit(Instruction::Dup);
                        let key_idx = self.add_constant(Value::from_string(element.key.clone()));
                        self.emit(Instruction::LoadConst(key_idx));
                        self.emit(Instruction::GetProperty);
                        if let Some(default_expr) = &element.default_value {
                            let skip_default = self.instructions.len();
                            self.emit(Instruction::JumpIfNotUndefined(0));
                            self.emit(Instruction::Pop);
                            self.generate_expression(default_expr)?;
                            self.patch_jump(skip_default, self.instructions.len());
                        }
                        self.generate_destructuring_pattern(&element.value)?;
                    }
                }
                self.emit(Instruction::Pop);
            }
        }
        Ok(())
    }

    fn generate_destructuring_assignment_target(&mut self, pattern: &Expression) -> Result<()> {
        match pattern {
            Expression::Identifier(id) => {
                if let Some(local_idx) = self.resolve_local(id) {
                    self.emit(Instruction::StoreLocal(local_idx));
                } else {
                    self.emit(Instruction::StoreGlobal(id.clone()));
                }
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
                    let idx = self.add_constant(Value::from_string(name.clone()));
                    self.emit(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.emit(Instruction::Rot3Right);
                self.emit(Instruction::SetProperty);
            }
            Expression::ArrayLiteral { elements } => {
                for (i, element) in elements.iter().enumerate() {
                    match element {
                        Expression::SpreadElement { .. } => {
                            self.emit(Instruction::Dup);
                            let idx = self.add_constant(Value::from_string("slice".to_string()));
                            self.emit(Instruction::LoadConst(idx));
                            let start_idx = self.add_constant(Value::Integer(i as i64));
                            self.emit(Instruction::LoadConst(start_idx));
                            self.emit(Instruction::CallMethod(1));
                            self.generate_destructuring_assignment_target(element)?;
                        }
                        _ => {
                            self.emit(Instruction::Dup);
                            let idx = self.add_constant(Value::Integer(i as i64));
                            self.emit(Instruction::LoadConst(idx));
                            self.emit(Instruction::GetProperty);
                            self.generate_destructuring_assignment_target(element)?;
                        }
                    }
                }
                self.emit(Instruction::Pop);
            }
            Expression::ObjectLiteral { properties } => {
                for prop in properties {
                    self.emit(Instruction::Dup);
                    if prop.computed {
                        if let Some(key_expr) = &prop.computed_key {
                            self.generate_expression(key_expr)?;
                        }
                    } else {
                        let key_idx = self.add_constant(Value::from_string(prop.key.clone()));
                        self.emit(Instruction::LoadConst(key_idx));
                    }
                    self.emit(Instruction::GetProperty);
                    self.generate_destructuring_assignment_target(&prop.value)?;
                }
                self.emit(Instruction::Pop);
            }
            _ => {
                self.emit(Instruction::Pop);
            }
        }
        Ok(())
    }

    fn extract_identifier_from_pattern(pattern: &BindingPattern) -> Option<String> {
        match pattern {
            BindingPattern::Identifier(name) => Some(name.clone()),
            _ => None,
        }
    }

    fn current_local_slot(&self) -> u16 {
        (self.captured_var_names.len() + self.locals.len() - self.local_start_idx) as u16
    }

    fn last_local_slot(&self) -> u16 {
        (self.captured_var_names.len() + self.locals.len() - 1 - self.local_start_idx) as u16
    }

    /// Record how many local slots this function needs. Must be called before
    /// truncating `locals` / restoring `captured_var_names` at end of compile.
    pub(crate) fn finalize_local_count(&mut self, func_idx: u32) {
        let count =
            self.captured_var_names.len() + self.locals.len().saturating_sub(self.local_start_idx);
        if let Some(fi) = self.functions.get_mut(func_idx as usize) {
            fi.local_count = count;
        }
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
            Instruction::JumpIfUndefined(addr) => *addr = target_u32,
            Instruction::JumpIfNotUndefined(addr) => *addr = target_u32,
            Instruction::Jump(addr) => *addr = target_u32,
            _ => {}
        }
    }
}
