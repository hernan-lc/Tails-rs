use super::CodeGenerator;
use crate::compiler::parser::{Expression, ForInit, SpannedNode, Statement};
use crate::compiler::CompiledFunction;
use crate::errors::Result;
use std::collections::HashSet;

impl CodeGenerator {
    pub(crate) fn compile_function(
        &mut self,
        name: Option<String>,
        params: &[String],
        body: &[SpannedNode<Statement>],
        is_generator: bool,
    ) -> Result<u32> {
        let func_idx = self.functions.len() as u32;
        let parent_locals_snapshot = self.locals.clone();
        let outer_refs = find_outer_refs(body, params, &parent_locals_snapshot);
        let num_captures = outer_refs.len();

        self.functions.push(CompiledFunction {
            name: name.clone(),
            params: params.to_vec(),
            rest_param: None,
            bytecode_index: 0,
            param_count: params.len(),
            closure_var_count: num_captures,
            is_generator,
            source_line: self.current_source_line,
            is_arrow: false,
        });

        let jump_over = self.instructions.len();
        self.emit(crate::compiler::Instruction::Jump(0));

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

        // JavaScript hoisting: pre-register all function names first
        for stmt in body {
            if let Statement::FunctionDeclaration { name, .. } = &stmt.inner {
                self.locals.push(name.clone());
            }
        }
        // JavaScript hoisting: compile function declarations first
        for stmt in body {
            if let Statement::FunctionDeclaration { .. } = &stmt.inner {
                self.record_line_from_span(&stmt.span);
                self.generate_statement(&stmt.inner, false)?;
            }
        }
        // Then compile non-function statements
        for stmt in body {
            if matches!(&stmt.inner, Statement::FunctionDeclaration { .. }) {
                continue;
            }
            self.record_line_from_span(&stmt.span);
            self.generate_statement(&stmt.inner, false)?;
        }

        self.emit(crate::compiler::Instruction::LoadUndefined);
        self.emit(crate::compiler::Instruction::Return);

        self.scope_depth -= 1;
        self.locals.truncate(prev_locals);
        self.captured_var_names = saved_captured;
        self.local_start_idx = saved_start;

        self.patch_jump(jump_over, self.instructions.len());
        Ok(func_idx)
    }
}

pub(crate) fn find_outer_refs(
    body: &[SpannedNode<Statement>],
    inner_params: &[String],
    parent_locals: &[String],
) -> Vec<(String, u16)> {
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

pub(crate) fn collect_identifiers_body(body: &[SpannedNode<Statement>], out: &mut Vec<String>) {
    for stmt in body {
        collect_identifiers_stmt(&stmt.inner, out);
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
        Statement::IfStatement {
            condition,
            consequent,
            alternate,
        } => {
            collect_identifiers_expr(condition, out);
            collect_identifiers_stmt(&consequent.inner, out);
            if let Some(alt) = alternate {
                collect_identifiers_stmt(&alt.inner, out);
            }
        }
        Statement::WhileStatement { condition, body } => {
            collect_identifiers_expr(condition, out);
            collect_identifiers_stmt(&body.inner, out);
        }
        Statement::BlockStatement(stmts) => {
            for s in stmts {
                collect_identifiers_stmt(&s.inner, out);
            }
        }
        Statement::ForStatement {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(for_init) = init {
                match for_init.as_ref() {
                    ForInit::Variable(stmt) => collect_identifiers_stmt(&stmt.inner, out),
                    ForInit::Expression(expr) => collect_identifiers_expr(expr, out),
                }
            }
            if let Some(cond) = condition {
                collect_identifiers_expr(cond, out);
            }
            if let Some(upd) = update {
                collect_identifiers_expr(upd, out);
            }
            collect_identifiers_stmt(&body.inner, out);
        }
        Statement::ForInStatement { right, body, .. }
        | Statement::ForOfStatement { right, body, .. } => {
            collect_identifiers_expr(right, out);
            collect_identifiers_stmt(&body.inner, out);
        }
        Statement::DoWhileStatement { condition, body } => {
            collect_identifiers_expr(condition, out);
            collect_identifiers_stmt(&body.inner, out);
        }
        Statement::SwitchStatement {
            discriminant,
            cases,
        } => {
            collect_identifiers_expr(discriminant, out);
            for case in cases {
                if let Some(test) = &case.test {
                    collect_identifiers_expr(test, out);
                }
                collect_identifiers_body(&case.consequent, out);
            }
        }
        Statement::ThrowStatement(expr) => collect_identifiers_expr(expr, out),
        Statement::TryStatement {
            block,
            handler,
            finalizer,
        } => {
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
        Expression::Member {
            object, property, ..
        } => {
            collect_identifiers_expr(object, out);
            collect_identifiers_expr(property, out);
        }
        Expression::OptionalMember {
            object, property, ..
        } => {
            collect_identifiers_expr(object, out);
            collect_identifiers_expr(property, out);
        }
        Expression::OptionalCall { callee, args } => {
            collect_identifiers_expr(callee, out);
            for arg in args {
                collect_identifiers_expr(arg, out);
            }
        }
        Expression::ConditionalExpression {
            test,
            consequent,
            alternate,
        } => {
            collect_identifiers_expr(test, out);
            collect_identifiers_expr(consequent, out);
            collect_identifiers_expr(alternate, out);
        }
        Expression::UpdateExpression { operand, .. } => {
            collect_identifiers_expr(operand, out);
        }
        Expression::ArrowFunction { body, .. } => match body.as_ref() {
            crate::compiler::parser::ArrowFunctionBody::Expression(expr) => {
                collect_identifiers_expr(expr, out)
            }
            crate::compiler::parser::ArrowFunctionBody::Block(stmts) => {
                collect_identifiers_body(stmts, out)
            }
        },
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
