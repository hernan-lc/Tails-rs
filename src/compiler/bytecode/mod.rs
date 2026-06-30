mod closures;
mod expressions;
mod stmt_control_flow;

use crate::compiler::parser::{
    ArrayBindingElement, ArrowFunctionBody, AstNode, BinaryOperator, BindingPattern, ClassMember,
    CompoundAssignmentOp, ExportDeclarationKind, Expression, SpannedNode, Statement,
    UnaryOperator, UpdateOperator,
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
        }
    }

    fn emit(&mut self, instr: Instruction) {
        self.instructions.push(instr);
        self.source_lines.push(self.current_source_line);
        self.source_cols.push(self.current_source_col);
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
                for (i, stmt) in statements.iter().enumerate() {
                    let is_last = i == statements.len() - 1;
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
        match stmt {
            Statement::Expression(expr) => {
                let is_assignment = matches!(expr, Expression::Assignment { .. });
                self.generate_expression(expr)?;
                if !is_last && !is_assignment {
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
                        self.generate_expression(init)?;
                        self.generate_destructuring_pattern(&decl.id)?;
                    } else {
                        match &decl.id {
                            BindingPattern::Identifier(id) => {
                                self.emit(Instruction::LoadUndefined);
                                if self.scope_depth == 0 {
                                    self.emit(Instruction::StoreGlobal(id.clone()));
                                } else {
                                    self.locals.push(id.clone());
                                    let slot = (self.locals.len() - 1 - self.local_start_idx) as u16;
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
                Ok(())
            }
            Statement::ReturnStatement(value) => {
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
                    let slot = (self.locals.len() - 1 - self.local_start_idx) as u16;
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
                Ok(())
            }
            Statement::ThrowStatement(argument) => {
                self.generate_expression(argument)?;
                self.emit(Instruction::Throw);
                Ok(())
            }
            Statement::ClassDeclaration {
                name,
                superclass,
                body,
            } => {
                let class_name = name.clone();

                let class_info_idx = self.class_infos.len() as u32;

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
                    name: class_name.clone(),
                    constructor_func_idx,
                    methods,
                    superclass: superclass_name,
                });

                if superclass.is_some() {
                    self.generate_expression(superclass.as_ref().unwrap())?;
                }

                self.emit(Instruction::MakeClass(class_info_idx));

                if self.scope_depth == 0 {
                    self.emit(Instruction::StoreGlobal(class_name));
                } else {
                    self.locals.push(class_name);
                    let slot = (self.locals.len() - 1 - self.local_start_idx) as u16;
                    self.emit(Instruction::StoreLocal(slot));
                }
                Ok(())
            }
            Statement::ImportDeclaration { specifiers, source } => {
                let is_native = source.ends_with(".native");
                if is_native {
                    if specifiers.len() == 1 {
                        let local_name = specifiers[0].local.clone();
                        self.emit(Instruction::NativeImport(source.clone(), local_name));
                    } else if specifiers.is_empty() {
                        self.emit(Instruction::NativeImport(
                            source.clone(),
                            "__module".to_string(),
                        ));
                    } else {
                        for spec in specifiers {
                            let local_name = spec.local.clone();
                            self.emit(Instruction::NativeImport(source.clone(), local_name));
                        }
                    }
                } else if specifiers.is_empty() {
                    self.emit(Instruction::ImportModule(source.clone()));
                } else if specifiers.len() == 1 && specifiers[0].imported.as_deref() == Some("*") {
                    self.emit(Instruction::ImportAll(
                        source.clone(),
                        specifiers[0].local.clone(),
                    ));
                } else {
                    for spec in specifiers {
                        let imported_name =
                            spec.imported.clone().unwrap_or_else(|| spec.local.clone());
                        if imported_name == "default" {
                            self.emit(Instruction::ImportDefault(
                                source.clone(),
                                spec.local.clone(),
                            ));
                        } else {
                            self.emit(Instruction::ImportNamed(
                                source.clone(),
                                imported_name,
                                spec.local.clone(),
                            ));
                        }
                    }
                }
                Ok(())
            }
            Statement::ExportDeclaration { kind } => {
                match kind {
                    ExportDeclarationKind::Local(declaration) => {
                        self.record_line_from_span(&declaration.span);
                        match &declaration.inner {
                            Statement::VariableDeclaration { declarations, .. } => {
                                let names: Vec<String> = declarations
                                    .iter()
                                    .filter_map(|d| Self::extract_identifier_from_pattern(&d.id))
                                    .collect();
                                self.generate_statement(&declaration.inner, false)?;
                                for name in &names {
                                    self.emit(Instruction::StoreModuleExport(name.clone()));
                                }
                            }
                            Statement::FunctionDeclaration { name, .. } => {
                                self.generate_statement(&declaration.inner, false)?;
                                self.emit(Instruction::StoreModuleExport(name.clone()));
                            }
                            Statement::ClassDeclaration { name, .. } => {
                                self.generate_statement(&declaration.inner, false)?;
                                self.emit(Instruction::StoreModuleExport(name.clone()));
                            }
                            _ => {
                                self.generate_statement(&declaration.inner, false)?;
                            }
                        }
                        Ok(())
                    }
                    ExportDeclarationKind::ReExport { specifiers, source } => {
                        if source.is_empty() {
                            // export { a, b } — re-export local names
                            for spec in specifiers {
                                let _exported_name = spec.exported.as_ref().unwrap_or(&spec.local);
                                let local_name = &spec.local;
                                self.emit(Instruction::StoreModuleExport(local_name.clone()));
                            }
                        } else if specifiers.len() == 1
                            && specifiers[0].local == "*"
                            && specifiers[0].exported.as_deref() == Some("*")
                        {
                            // export * from "source" — re-export all
                            self.emit(Instruction::ReExportAll(source.clone()));
                        } else if specifiers.len() == 1 && specifiers[0].local == "*" {
                            // export * as name from "source"
                            let alias = specifiers[0].exported.as_ref().unwrap();
                            self.emit(Instruction::ImportAll(source.clone(), alias.clone()));
                            self.emit(Instruction::StoreModuleExport(alias.clone()));
                        } else {
                            // Re-export: export { a as b } from "./module";
                            self.emit(Instruction::ImportModule(source.clone()));
                            for spec in specifiers {
                                let imported_name = spec.exported.as_ref().unwrap_or(&spec.local);
                                let local_name = &spec.local;
                                self.emit(Instruction::ImportNamed(
                                    source.clone(),
                                    imported_name.clone(),
                                    local_name.clone(),
                                ));
                                self.emit(Instruction::StoreModuleExport(local_name.clone()));
                            }
                        }
                        Ok(())
                    }
                }
            }
            Statement::InterfaceDeclaration { .. } => Ok(()),
            Statement::TypeAliasDeclaration { .. } => Ok(()),
            Statement::EnumDeclaration { .. } => Ok(()),
            Statement::ExportDefaultDeclaration { declaration } => {
                self.record_line_from_span(&declaration.span);
                match &declaration.inner {
                    Statement::FunctionDeclaration { name, .. } => {
                        self.generate_statement(&declaration.inner, false)?;
                        self.emit(Instruction::StoreModuleExport(name.clone()));
                        self.emit(Instruction::LoadGlobal(name.clone()));
                        self.emit(Instruction::ExportDefault);
                    }
                    Statement::ClassDeclaration { name, .. } => {
                        self.generate_statement(&declaration.inner, false)?;
                        self.emit(Instruction::StoreModuleExport(name.clone()));
                        self.emit(Instruction::LoadGlobal(name.clone()));
                        self.emit(Instruction::ExportDefault);
                    }
                    _ => {
                        self.generate_statement(&declaration.inner, true)?;
                        self.emit(Instruction::ExportDefault);
                    }
                }
                Ok(())
            }
            Statement::BlockStatement(stmts) => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();
                self.emit(Instruction::BlockEnter);
                for (i, stmt) in stmts.iter().enumerate() {
                    let is_last = i == stmts.len() - 1;
                    self.record_line_from_span(&stmt.span);
                    self.generate_statement(&stmt.inner, is_last)?;
                }
                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                }
                self.emit(Instruction::BlockExit);
                self.scope_depth -= 1;
                Ok(())
            }
            Statement::FunctionDeclaration {
                name,
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
                    closures::find_outer_refs(body, &all_params, &parent_locals_snapshot);
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
                Ok(())
            }
            _ => unreachable!("control flow statements handled by generate_control_flow_statement"),
        }
    }

    fn generate_statement_in_branch(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::BlockStatement(stmts) => {
                self.scope_depth += 1;
                let prev_locals_count = self.locals.len();
                self.emit(Instruction::BlockEnter);
                for (i, stmt) in stmts.iter().enumerate() {
                    let is_last = i == stmts.len() - 1;
                    self.record_line_from_span(&stmt.span);
                    self.generate_statement(&stmt.inner, is_last)?;
                }
                let locals_added = self.locals.len() - prev_locals_count;
                for _ in 0..locals_added {
                    self.locals.pop();
                }
                self.emit(Instruction::BlockExit);
                self.scope_depth -= 1;
                Ok(())
            }
            _ => self.generate_statement(stmt, true),
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
                    self.locals.push(id.clone());
                    let slot = (self.locals.len() - 1 - self.local_start_idx) as u16;
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
                            let idx = self.add_constant(Value::String("slice".to_string()));
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
                for element in elements {
                    self.emit(Instruction::Dup);
                    let key_idx = self.add_constant(Value::String(element.key.clone()));
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
                self.emit(Instruction::Pop);
            }
        }
        Ok(())
    }

    pub(crate) fn compile_class_constructor(
        &mut self,
        body: &[ClassMember],
    ) -> Result<Option<u32>> {
        // Collect property initializers
        let mut prop_inits: Vec<(String, Option<Expression>)> = Vec::new();
        for member in body {
            if let ClassMember::Property { name, init, .. } = member {
                prop_inits.push((name.clone(), init.clone()));
            }
        }

        // Find explicit constructor
        for member in body {
            if let ClassMember::Constructor { params, body } = member {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                // If there are property initializers, we need to prepend them to the constructor body
                if prop_inits.is_empty() {
                    return Ok(Some(self.compile_function(
                        Some("constructor".to_string()),
                        &param_names,
                        body,
                        false,
                    )?));
                }
                // Build synthetic statements for property initializations
                let mut synthetic_body: Vec<SpannedNode<Statement>> = Vec::new();
                for (name, init_expr) in &prop_inits {
                    let value = if let Some(expr) = init_expr {
                        expr.clone()
                    } else {
                        Expression::UndefinedLiteral
                    };
                    synthetic_body.push(SpannedNode {
                        inner: Statement::Expression(Expression::Assignment {
                            target: Box::new(Expression::Member {
                                object: Box::new(Expression::Identifier("this".to_string())),
                                property: Box::new(Expression::Identifier(name.clone())),
                                computed: false,
                            }),
                            value: Box::new(value),
                            op: None,
                        }),
                        span: Some(crate::errors::Span::unknown()),
                    });
                }
                synthetic_body.extend(body.clone());
                return Ok(Some(self.compile_function(
                    Some("constructor".to_string()),
                    &param_names,
                    &synthetic_body,
                    false,
                )?));
            }
        }

        // No explicit constructor but has property initializers — create a default constructor
        if !prop_inits.is_empty() {
            let mut synthetic_body: Vec<SpannedNode<Statement>> = Vec::new();
            for (name, init_expr) in &prop_inits {
                let value = if let Some(expr) = init_expr {
                    expr.clone()
                } else {
                    Expression::UndefinedLiteral
                };
                synthetic_body.push(SpannedNode {
                    inner: Statement::Expression(Expression::Assignment {
                        target: Box::new(Expression::Member {
                            object: Box::new(Expression::Identifier("this".to_string())),
                            property: Box::new(Expression::Identifier(name.clone())),
                            computed: false,
                        }),
                        value: Box::new(value),
                        op: None,
                    }),
                    span: Some(crate::errors::Span::unknown()),
                });
            }
            return Ok(Some(self.compile_function(
                Some("constructor".to_string()),
                &[],
                &synthetic_body,
                false,
            )?));
        }

        Ok(None)
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
