use super::*;

impl Interpreter {
    pub(crate) fn execute_from(
        &mut self,
        module: &CompiledModule,
        start_pc: usize,
    ) -> Result<Value> {
        let mut pc = start_pc;

        loop {
            if pc >= module.instructions.len() {
                break;
            }

            self.current_pc = pc;

            if pc & 127 == 0 && self.gc.should_collect() {
                self.collect_garbage();
            }

            let instruction = &module.instructions[pc];

            if cfg!(debug_assertions) && std::env::var("GEN_TRACE").is_ok() {
                eprintln!("[GEN_TRACE] pc={}, instr={:?}", pc, instruction);
            }

            match instruction {
                // OPTIMIZATION (Phase 1C): Hot-path instructions are inlined
                // directly in the main match to skip the cascading
                // `_ => exec_load_store()` dispatch (one branch per iter).
                Instruction::LoadLocal(slot) => {
                    if let Some(frame) = self.call_stack.last() {
                        let idx = frame.base_pointer + *slot as usize;
                        if let Some(value) = self.stack.get(idx) {
                            self.stack.push(value.clone());
                            pc += 1;
                            continue;
                        }
                    } else if let Some(value) = self.stack.get(*slot as usize) {
                        self.stack.push(value.clone());
                        pc += 1;
                        continue;
                    }
                    self.stack.push(Value::Undefined);
                    pc += 1;
                    continue;
                }
                Instruction::StoreLocal(slot) => {
                    if let Some(value) = self.stack.pop() {
                        let base = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                        let idx = base + *slot as usize;
                        if idx >= self.stack.len() {
                            self.stack.resize(idx + 1, Value::Undefined);
                        }
                        self.stack[idx] = value;
                    } else {
                        return Err(self.err_at_location(Error::RuntimeError(
                            super::ERR_STACK_UNDERFLOW.into(),
                        )));
                    }
                    pc += 1;
                    continue;
                }
                Instruction::IncLocal(slot, delta) => {
                    if let Some(frame) = self.call_stack.last() {
                        let idx = frame.base_pointer + *slot as usize;
                        if idx < self.stack.len() {
                            match &self.stack[idx] {
                                Value::Integer(n) => {
                                    self.stack[idx] = Value::Integer(n + delta);
                                    pc += 1;
                                    continue;
                                }
                                Value::Float(n) => {
                                    self.stack[idx] = Value::Float(n + *delta as f64);
                                    pc += 1;
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                    // Cold path: fall through to exec_load_store.
                }
                Instruction::AddLocal(dst, src) => {
                    let base = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                    let dst_idx = base + *dst as usize;
                    let src_idx = base + *src as usize;
                    if dst_idx < self.stack.len() && src_idx < self.stack.len() {
                        match (&self.stack[dst_idx], &self.stack[src_idx]) {
                            (Value::Integer(a), Value::Integer(b)) => {
                                if let Some(result) = a.checked_add(*b) {
                                    self.stack[dst_idx] = Value::Integer(result);
                                } else {
                                    self.stack[dst_idx] = Value::Float(*a as f64 + *b as f64);
                                }
                                pc += 1;
                                continue;
                            }
                            (Value::Float(a), Value::Float(b)) => {
                                self.stack[dst_idx] = Value::Float(a + b);
                                pc += 1;
                                continue;
                            }
                            (Value::Integer(a), Value::Float(b)) => {
                                self.stack[dst_idx] = Value::Float(*a as f64 + *b);
                                pc += 1;
                                continue;
                            }
                            (Value::Float(a), Value::Integer(b)) => {
                                self.stack[dst_idx] = Value::Float(*a + *b as f64);
                                pc += 1;
                                continue;
                            }
                            _ => {
                                // Phase 1.8: Move values out to avoid borrow
                                // conflicts, then handle string/cons/cold paths.
                                let left =
                                    std::mem::replace(&mut self.stack[dst_idx], Value::Undefined);
                                let right =
                                    std::mem::replace(&mut self.stack[src_idx], Value::Undefined);
                                match (&left, &right) {
                                    (Value::String(a), Value::String(b)) => {
                                        self.stack[dst_idx] = Value::Cons(ConsString::new_smart(
                                            Value::String(a.clone()),
                                            Value::String(b.clone()),
                                        ));
                                    }
                                    (Value::Cons(_), Value::String(_))
                                    | (Value::String(_), Value::Cons(_))
                                    | (Value::Cons(_), Value::Cons(_)) => {
                                        self.stack[dst_idx] =
                                            Value::Cons(ConsString::new(left, right));
                                    }
                                    _ => {
                                        self.stack[dst_idx] = self.add(left, right)?;
                                    }
                                }
                                pc += 1;
                                continue;
                            }
                        }
                    }
                }
                // Fused loop branch: increment counter, compare with limit, jump
                Instruction::LoopBranch {
                    counter_slot,
                    limit_const,
                    body_pc,
                    step,
                } => {
                    if let Some(frame) = self.call_stack.last() {
                        let idx = frame.base_pointer + *counter_slot as usize;
                        if idx < self.stack.len() {
                            // Increment counter
                            match &self.stack[idx] {
                                Value::Integer(n) => {
                                    let new_val = n.wrapping_add(*step);
                                    self.stack[idx] = Value::Integer(new_val);
                                }
                                Value::Float(n) => {
                                    self.stack[idx] = Value::Float(n + *step as f64);
                                }
                                _ => {}
                            }
                            // Compare with limit
                            let counter_val = &self.stack[idx];
                            let cidx = *limit_const as usize;
                            if let Some(limit_val) = module.constants.get(cidx) {
                                let should_continue = match (counter_val, limit_val) {
                                    (Value::Integer(a), Value::Integer(b)) => a < b,
                                    (Value::Integer(a), Value::Float(b)) => (*a as f64) < *b,
                                    (Value::Float(a), Value::Integer(b)) => *a < (*b as f64),
                                    (Value::Float(a), Value::Float(b)) => a < b,
                                    _ => self.less_than(counter_val, limit_val)?,
                                };
                                if should_continue {
                                    // Phase 8.10: Profile loop back-edges for
                                    // JIT compilation.  Tick every 128
                                    // iterations to amortize profiler overhead.
                                    match counter_val {
                                        Value::Integer(n) if n & 127 == 0 => {
                                            self.jit.tick(pc, module);
                                        }
                                        _ => {}
                                    }
                                    pc = *body_pc as usize;
                                    continue;
                                } else {
                                    pc += 1;
                                    continue;
                                }
                            }
                        }
                    }
                    // Cold path: fall through
                }
                // Fused global add: x = x + local. Read globals by ref (no clone)
                // on the integer/float hot paths.
                Instruction::AddGlobal(name, local_slot) => {
                    if let Some(frame) = self.call_stack.last() {
                        let local_idx = frame.base_pointer + *local_slot as usize;
                        if local_idx < self.stack.len() {
                            let right = self.stack[local_idx].clone();
                            let left_ref = self.globals.get(name.as_str());
                            match (left_ref, &right) {
                                (Some(Value::Integer(a)), Value::Integer(b)) => {
                                    if let Some(result) = a.checked_add(*b) {
                                        self.globals.insert(name.clone(), Value::Integer(result));
                                    } else {
                                        self.globals.insert(
                                            name.clone(),
                                            Value::Float(*a as f64 + *b as f64),
                                        );
                                    }
                                    pc += 1;
                                    continue;
                                }
                                (Some(Value::Float(a)), Value::Float(b)) => {
                                    self.globals.insert(name.clone(), Value::Float(a + b));
                                    pc += 1;
                                    continue;
                                }
                                (Some(Value::Integer(a)), Value::Float(b)) => {
                                    self.globals
                                        .insert(name.clone(), Value::Float(*a as f64 + *b));
                                    pc += 1;
                                    continue;
                                }
                                (Some(Value::Float(a)), Value::Integer(b)) => {
                                    self.globals
                                        .insert(name.clone(), Value::Float(*a + *b as f64));
                                    pc += 1;
                                    continue;
                                }
                                _ => {
                                    let left = left_ref.cloned().unwrap_or(Value::Undefined);
                                    let result = self.add(left, right)?;
                                    self.globals.insert(name.clone(), result);
                                    pc += 1;
                                    continue;
                                }
                            }
                        }
                    }
                    // Cold path: fall through
                }
                // Phase 8.3: Inline LoadGlobal / StoreGlobal on the hot path
                // to avoid cascading dispatch for these very common instructions.
                Instruction::LoadGlobal(name) => {
                    let val = self.globals.get(name.as_str()).cloned().or_else(|| {
                        self.module_globals
                            .as_ref()
                            .and_then(|mg| mg.borrow().get(name.as_str()).cloned())
                    });
                    match val {
                        Some(v) => {
                            self.stack.push(v);
                        }
                        None => {
                            return Err(self.err_at_location(Error::ReferenceError(format!(
                                "{} is not defined",
                                name
                            ))));
                        }
                    }
                    pc += 1;
                    continue;
                }
                Instruction::StoreGlobal(name) => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
                    if self.module_globals.is_some() {
                        if let Some(ref mg) = self.module_globals {
                            mg.borrow_mut().insert(name.clone(), value.clone());
                        }
                        self.globals.insert(name.clone(), value);
                    } else {
                        self.globals.insert(name.clone(), value);
                    }
                    pc += 1;
                    continue;
                }
                // Phase 8.4: Inline MapSet/MapGet. Fast path for the common
                // 2-arg form pops value/key/map without a temporary args Vec.
                Instruction::MapSet(argc) => {
                    if *argc == 2 {
                        let value = self.stack.pop().ok_or_else(|| {
                            Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                        })?;
                        let key = self.stack.pop().ok_or_else(|| {
                            Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                        })?;
                        let object = self.stack.pop().ok_or_else(|| {
                            Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                        })?;
                        if let Value::Map(map_idx) = object {
                            if let HeapValue::Map(map) = &mut self.heap[map_idx] {
                                map.set(key, value);
                            }
                            self.stack.push(Value::Map(map_idx));
                        } else {
                            let method =
                                self.get_property(&object, &Value::string("set"))?;
                            let result =
                                self.call_value(&method, &object, &[key, value])?;
                            self.stack.push(result);
                        }
                    } else {
                        let mut args = Vec::with_capacity(usize::from(*argc));
                        for _ in 0..*argc {
                            args.push(self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?);
                        }
                        args.reverse();
                        let object = self.stack.pop().ok_or_else(|| {
                            Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                        })?;
                        if let Value::Map(map_idx) = object {
                            let key = args.first().cloned().unwrap_or(Value::Undefined);
                            let value = args.get(1).cloned().unwrap_or(Value::Undefined);
                            if let HeapValue::Map(map) = &mut self.heap[map_idx] {
                                map.set(key, value);
                            }
                            self.stack.push(Value::Map(map_idx));
                        } else {
                            let method =
                                self.get_property(&object, &Value::string("set"))?;
                            let result = self.call_value(&method, &object, &args)?;
                            self.stack.push(result);
                        }
                    }
                    pc += 1;
                    continue;
                }
                Instruction::MapGet => {
                    let key = self.stack.pop().ok_or_else(|| {
                        Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                    })?;
                    let object = self.stack.pop().ok_or_else(|| {
                        Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                    })?;
                    if let Value::Map(map_idx) = object {
                        let result = if let HeapValue::Map(map) = &self.heap[map_idx] {
                            map.get(&key).cloned().unwrap_or(Value::Undefined)
                        } else {
                            Value::Undefined
                        };
                        self.stack.push(result);
                    } else {
                        let method = self.get_property(&object, &Value::string("get"))?;
                        let result = self.call_value(&method, &object, &[key])?;
                        self.stack.push(result);
                    }
                    pc += 1;
                    continue;
                }
                // OPTIMIZATION (Phase 1G): Inline the most common `LoadConst`
                // pattern (Integer / Float / small String / Null / Undefined /
                // Boolean) directly in the dispatch so the cascading
                // `_ => exec_load_store()` branch is skipped. The cold path
                // (Object / Array / Function / …) delegates to
                // `exec_load_store` explicitly so the value is actually pushed
                // — a fall-through would skip the push and cause a Stack
                // underflow on the next consumer (this was a real bug in an
                // earlier version of this optimisation).
                //
                // Phase 1.3: `BigInt` and `Symbol` are also immediate values
                // (16 bytes / 8 bytes respectively) so we push a clone inline
                // instead of going through `exec_load_store`. The clone is
                // cheap (one discriminant + payload memcpy) and the call
                // savings are real for code that uses `const BIG = 100n` /
                // `const S = Symbol("x")` in a hot loop.
                Instruction::LoadConst(idx) => {
                    let cidx = *idx as usize;
                    if let Some(value) = module.constants.get(cidx) {
                        match value {
                            Value::Integer(_)
                            | Value::Float(_)
                            | Value::String(_)
                            | Value::Null
                            | Value::Undefined
                            | Value::Boolean(_)
                            | Value::BigInt(_)
                            | Value::Symbol(_) => {
                                self.stack.push(value.clone());
                                pc += 1;
                                continue;
                            }
                            _ => {
                                // Object, Array, Function, … — fall through
                                // to `exec_load_store` below.
                            }
                        }
                    }
                    // Cold path: explicitly call `exec_load_store` so the
                    // value (a BigInt / Object / …) is actually pushed.
                    if self.exec_load_store(instruction, module)? {
                        // exec_load_store returned true — it handled the
                        // instruction. Fall through to `pc += 1` below.
                    }
                }
                // OPTIMIZATION (Phase 1G): `Pop` is one of the most frequent
                // instructions after every expression statement. Inline it
                // so the dispatch is one branch + one pop.
                Instruction::Pop => {
                    self.stack.pop();
                    pc += 1;
                    continue;
                }
                // OPTIMIZATION (Phase 1H): `Dup` (duplicate the top of the
                // stack) is emitted for every `i++` / `++i` and for compound
                // assignments like `x += y` after the right-hand side is
                // pushed. Inlining avoids the cascading `_ =>
                // exec_load_store()` dispatch (one branch per iter).
                Instruction::Dup => {
                    let val = self.stack.last().cloned().unwrap_or(Value::Undefined);
                    self.stack.push(val);
                    pc += 1;
                    continue;
                }
                // OPTIMIZATION (Phase 1H): `LoadThis` is emitted at the start
                // of every non-arrow function body and on `super.X` accesses.
                // Inlining skips the call to `exec_load_store` and avoids the
                // `Option::clone()` of `f.this_value` (which the existing
                // path in `exec_load_store` does unconditionally even when
                // it's `None`).
                Instruction::LoadThis => {
                    let this = self
                        .call_stack
                        .last()
                        .and_then(|f| f.this_value.clone())
                        .unwrap_or(Value::Undefined);
                    self.stack.push(this);
                    pc += 1;
                    continue;
                }
                // OPTIMIZATION (Phase 1H): `Rot3Right` rotates the top three
                // stack values (used by short-circuit boolean and ternary
                // operators). Inlining avoids the function-call round-trip
                // and the 3 × 32-byte clones the old `exec_load_store`
                // arm did. The rotation is `(a, b, c) → (b, c, a)`:
                //   - stack[len-3] should hold b (the second)
                //   - stack[len-2] should hold c (the third)
                //   - stack[len-1] should hold a (the first)
                // We achieve this with three `std::mem::replace` moves
                // (no clones) plus one direct write. The order matters
                // because each replace must read its slot BEFORE it is
                // overwritten by the next step.
                Instruction::Rot3Right => {
                    let len = self.stack.len();
                    if len >= 3 {
                        // Step 1: read `a` out of stack[len-3] (replacing
                        //         it with Undefined), put `a` into
                        //         stack[len-1] (replacing the original
                        //         `c` and reading it out as `c`).
                        let a = std::mem::replace(&mut self.stack[len - 3], Value::Undefined);
                        let c = std::mem::replace(&mut self.stack[len - 1], a);
                        // Step 2: read `b` out of stack[len-2] (replacing
                        //         it with the original `c`).
                        let b = std::mem::replace(&mut self.stack[len - 2], c);
                        // Step 3: write `b` into stack[len-3] (the slot
                        //         that currently holds Undefined).
                        self.stack[len - 3] = b;
                        // Final: stack[len-3] = b, stack[len-2] = c,
                        //        stack[len-1] = a  →  (b, c, a) ✓
                    }
                    pc += 1;
                    continue;
                }
                Instruction::Jump(_)
                | Instruction::JumpIf(_)
                | Instruction::JumpIfNot(_)
                | Instruction::JumpIfUndefined(_)
                | Instruction::JumpIfNotUndefined(_)
                | Instruction::Return
                | Instruction::Yield => {
                    let mut pc_mut = pc;
                    match self.exec_control_flow(instruction, module, &mut pc_mut)? {
                        ControlFlowOutcome::Continue => {
                            pc = pc_mut;
                            continue;
                        }
                        ControlFlowOutcome::Return(v) => return Ok(v),
                        ControlFlowOutcome::Next => {}
                        ControlFlowOutcome::Jump(target) => {
                            pc = target;
                            continue;
                        }
                    }
                }
                Instruction::Call(argc) => {
                    if self.exec_call(argc, module, &mut pc)? {
                        continue;
                    }
                }
                Instruction::CallMethod(argc) => {
                    if self.exec_call_method(argc, &mut pc)? {
                        continue;
                    }
                }
                Instruction::Construct(argc) => {
                    if self.exec_construct(argc, module, &mut pc)? {
                        continue;
                    }
                }
                Instruction::ConstructApply => {
                    if self.exec_construct_apply(module, &mut pc)? {
                        continue;
                    }
                }
                Instruction::MakeClass(_)
                | Instruction::SuperConstruct(_)
                | Instruction::SuperGet => {
                    let mut pc_mut = pc;
                    self.exec_class_ops(instruction, &mut pc_mut, module)?;
                    if pc_mut != pc {
                        pc = pc_mut;
                        continue;
                    }
                }
                Instruction::ImportModule(source) => {
                    let module_obj = self.exec_import_module(source)?.unwrap_or(Value::Undefined);
                    self.stack.push(module_obj);
                }
                Instruction::ImportNamed(source, imported_name, local_name) => {
                    let _ = self.exec_import_named(source, imported_name, local_name)?;
                }
                Instruction::ImportDefault(source, local_name) => {
                    let _ = self.exec_import_default(source, local_name)?;
                }
                Instruction::ImportAll(source, local_name) => {
                    let _ = self.exec_import_all(source, local_name)?;
                }
                Instruction::NativeImport(source, local_name) => {
                    self.exec_native_import(source, local_name)?;
                }
                Instruction::ExportNamed(names) => {
                    self.exec_export_named(names.as_slice())?;
                }
                Instruction::ExportDefault => {
                    self.exec_export_default()?;
                }
                Instruction::StoreModuleExport(name) => {
                    self.exec_store_module_export(name)?;
                }
                Instruction::ReExportAll(source) => {
                    self.exec_reexport_all(source)?;
                }
                Instruction::PopModuleExports => {
                    self.module_exports.clear();
                }
                Instruction::Await => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
                    if let Value::Promise(promise_idx) = &value {
                        if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                            match &p.state {
                                PromiseState::Fulfilled(v) => {
                                    self.stack.push(v.clone());
                                }
                                PromiseState::Rejected(reason) => {
                                    self.pending_exception = Some(reason.clone());
                                    if self.handle_pending_exception(&mut pc)? {
                                        continue;
                                    }
                                }
                                PromiseState::Pending => {
                                    let frame = SuspendedFrame {
                                        promise_idx: *promise_idx,
                                        resume_pc: pc + 1,
                                        stack_snapshot: std::mem::take(&mut self.stack),
                                        call_stack_snapshot: std::mem::take(&mut self.call_stack),
                                        module: self.current_module.clone(),
                                        module_path: self.current_module_path.clone(),
                                        exception_handlers_snapshot: std::mem::take(
                                            &mut self.exception_handlers,
                                        ),
                                        block_scope_stack_snapshot: std::mem::take(
                                            &mut self.block_scope_stack,
                                        ),
                                    };
                                    self.suspended_frames.push_back(frame);
                                    return Ok(Value::Undefined);
                                }
                            }
                        } else {
                            self.stack.push(value);
                        }
                    } else {
                        self.stack.push(value);
                    }
                }
                Instruction::DynamicImport => {
                    let source = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
                    let source_str = match &source {
                        Value::String(s) => s.clone(),
                        _ => {
                            let promise_idx = self.heap.len();
                            self.heap.push(HeapValue::Promise(
                                crate::objects::js_promise::JsPromise::rejected(Value::from_string(format!("Cannot resolve import source: {}", source),)),
                            ));
                            self.stack.push(Value::Promise(promise_idx));
                            continue;
                        }
                    };
                    match self.load_and_run_module(&source_str) {
                        Ok(Some(module_path)) => {
                            let exports = self
                                .module_registry
                                .get(&module_path)
                                .cloned()
                                .unwrap_or_default();
                            let promise = self.build_module_promise(exports);
                            self.stack.push(promise);
                        }
                        Ok(None) => {
                            let promise = self
                                .build_error_promise(format!("Module '{}' not found", source_str));
                            self.stack.push(promise);
                        }
                        Err(e) => {
                            let promise = self.build_error_promise(format!(
                                "Module '{}' error: {}",
                                source_str, e
                            ));
                            self.stack.push(promise);
                        }
                    }
                }
                Instruction::GetIterator => {
                    let iterable = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
                    let iter = self.exec_get_iterator(iterable)?;
                    self.stack.push(iter);
                }
                Instruction::GetAsyncIterator => {
                    let iterable = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
                    let iter = self.exec_get_async_iterator(iterable)?;
                    self.stack.push(iter);
                }
                Instruction::IteratorNext(target) => {
                    let iterator =
                        self.stack.last().cloned().ok_or_else(|| {
                            Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                        })?;
                    match self.exec_iterator_next(iterator, *target as usize)? {
                        ControlFlowOutcome::Jump(jump_target) => {
                            self.stack.pop();
                            pc = jump_target;
                            continue;
                        }
                        ControlFlowOutcome::Next => {}
                        _ => {}
                    }
                }
                Instruction::IteratorClose => {
                    let iterator = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
                    self.exec_iterator_close(iterator)?;
                }
                Instruction::AsyncIteratorNext(target) => {
                    let iterator =
                        self.stack.last().cloned().ok_or_else(|| {
                            Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                        })?;
                    match self.exec_async_iterator_next(iterator, *target as usize)? {
                        ControlFlowOutcome::Jump(jump_target) => {
                            self.stack.pop();
                            pc = jump_target;
                            continue;
                        }
                        ControlFlowOutcome::Next => {}
                        _ => {}
                    }
                }
                _ => {
                    // Phase 6: Inline hottest instructions to avoid
                    // cascading dispatch overhead (each avoided function
                    // call saves ~5ns per instruction × millions of calls)
                    match instruction {
                        Instruction::Add => {
                            let right = self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?;
                            let left = self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?;
                            let result = self.add(left, right)?;
                            self.stack.push(result);
                            pc += 1;
                            continue;
                        }
                        Instruction::Sub => {
                            let right = self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?;
                            let left = self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?;
                            let result = self.sub(left, right)?;
                            self.stack.push(result);
                            pc += 1;
                            continue;
                        }
                        Instruction::Eq => {
                            let right = self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?;
                            let left = self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?;
                            self.stack
                                .push(Value::Boolean(self.is_equal(&left, &right)));
                            pc += 1;
                            continue;
                        }
                        Instruction::GetProperty => {
                            let key = self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?;
                            let object = self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?;
                            let result = self.get_property(&object, &key)?;
                            self.stack.push(result);
                            pc += 1;
                            continue;
                        }
                        Instruction::SetProperty => {
                            let value = self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?;
                            let key = self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?;
                            let object = self.stack.pop().ok_or_else(|| {
                                Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                            })?;
                            // Inline the fast path for Object properties, but
                            // only when no accessors (getters/setters) exist.
                            if let Value::Object(obj_idx) = &object {
                                if let Value::String(key_str) = &key {
                                    if let HeapValue::Object(obj) = &mut self.heap[*obj_idx] {
                                        if !obj.properties.has_accessors() {
                                            obj.properties.insert(key_str.to_string(), value);
                                            self.stack.push(object);
                                            pc += 1;
                                            continue;
                                        }
                                    }
                                }
                            }
                            // Fall through to full handler for setters/getters
                            // and non-Object types
                            self.stack.push(object);
                            self.stack.push(key);
                            self.stack.push(value);
                        }
                        _ => {}
                    }
                    #[allow(clippy::if_same_then_else)]
                    if self.exec_load_store(instruction, module)? {
                    } else if self.exec_arithmetic(instruction)? {
                    } else if self.exec_comparison(instruction)? {
                    } else if self.exec_property_ops(instruction)? {
                    } else if self.exec_make_function(instruction, module, pc)? {
                    } else if self.exec_class_ops(instruction, &mut pc, module)? {
                    } else {
                        let saved_pc = pc;
                        if self.exec_exception(instruction, &mut pc)? {
                            if pc != saved_pc {
                                continue;
                            }
                        } else if self.handle_pending_exception(&mut pc)? {
                            if pc != saved_pc {
                                continue;
                            }
                        } else {
                            return Err(Error::RuntimeError(format!(
                                "Unhandled instruction: {:?}",
                                instruction
                            )));
                        }
                    }
                }
            }

            pc += 1;
        }

        Ok(self.stack.pop().unwrap_or(Value::Undefined))
    }
}
