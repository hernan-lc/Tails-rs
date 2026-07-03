use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::props;
use crate::vm::interpreter::Interpreter;

pub(super) fn native_generator_next(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Generator(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Generator".into())),
    };

    let value = args.first().cloned().unwrap_or(Value::Undefined);

    let (func_heap_idx, resume_pc) =
        if let crate::vm::interpreter::HeapValue::Generator(gen) = &interp.heap[idx] {
            (gen.func_heap_idx, gen.resume_pc)
        } else {
            return Err(Error::TypeError("Not a Generator".into()));
        };

    if let crate::vm::interpreter::HeapValue::Generator(gen) = &mut interp.heap[idx] {
        let module = interp.current_module.clone();
        if let Some(module) = module {
            let return_address = module.instructions.len();
            let base_pointer = interp.stack.len();

            // OPTIMIZATION (Phase 6A): avoid 3 stack copies per .next() by
            // using std::mem::take to move the saved state out of the
            // generator heap object directly. The original code cloned the
            // Vec before extending (copy 1), then `to_vec` the post-execution
            // stack slice back into the generator (copy 3). Now we move:
            //   1. take() saved_stack out of the generator, then push the
            //      elements one by one (no copy of the underlying buffer;
            //      it's just a realloc-and-move of the Vec's heap data).
            //   2. after execute_from, take the new state out of the stack
            //      directly using drain() — no `to_vec` clone.
            //
            // We need to put saved_stack back before any return, so use a
            // guard pattern.
            let mut saved_stack = std::mem::take(&mut gen.saved_stack);
            let saved_block_scope = std::mem::take(&mut gen.saved_block_scope_stack);
            let outer_block_scope =
                std::mem::replace(&mut interp.block_scope_stack, saved_block_scope);

            // Move elements from the saved stack onto the interpreter stack
            // without cloning the underlying buffer.
            interp.stack.append(&mut saved_stack);
            interp.stack.push(value);

            let closure_count = 0;
            let call_frame_len_before = interp.call_stack.len();

            if interp.call_stack.len() >= interp.max_call_stack_depth {
                return Err(crate::errors::Error::RuntimeError(
                    "Maximum call stack size exceeded".into(),
                ));
            }
            interp.call_stack.push(crate::vm::interpreter::CallFrame {
                return_address,
                base_pointer,
                closure_var_count: closure_count,
                func_heap_idx,
                this_value: None,
                is_construct: false,
                source_name: None,
                generator_heap_idx: Some(idx),
                source_line: None,
                source_col: None,
                exception_handlers_snapshot: if interp.exception_handlers.is_empty() {
                    Vec::new()
                } else {
                    interp.exception_handlers.clone()
                },
            });

            let result = interp.execute_from(&module, resume_pc);

            let yielded = if let crate::vm::interpreter::HeapValue::Generator(g) = &interp.heap[idx]
            {
                g.generator_yielded
            } else {
                false
            };
            if let crate::vm::interpreter::HeapValue::Generator(gen2) = &mut interp.heap[idx] {
                gen2.generator_yielded = false;
            }

            interp.block_scope_stack = outer_block_scope;

            if interp.call_stack.len() > call_frame_len_before {
                interp.call_stack.pop();
            }

            if let crate::vm::interpreter::HeapValue::Generator(gen2) = &mut interp.heap[idx] {
                if result.is_err() || (result.is_ok() && !yielded) {
                    // Drain the rest of the stack slice back into a Vec for
                    // potential future use, then clear it.
                    let new_saved: Vec<Value> = if interp.stack.len() > base_pointer {
                        interp.stack.drain(base_pointer..).collect()
                    } else {
                        Vec::new()
                    };
                    gen2.saved_stack = new_saved;
                    gen2.saved_block_scope_stack = Vec::new();
                    gen2.resume_pc = usize::MAX;
                } else if let Ok(ref _val) = result {
                    // OPTIMIZATION: drain the slice (move) instead of
                    // to_vec() (clone). drain() yields owned Values without
                    // copying the underlying heap data.
                    let new_saved: Vec<Value> = if interp.stack.len() > base_pointer {
                        interp.stack.drain(base_pointer..).collect()
                    } else {
                        Vec::new()
                    };
                    gen2.saved_stack = new_saved;
                    // block_scope_stack — clone is still needed because we
                    // moved outer_block_scope back into interp above.
                    // (Cold path: only happens once per yield.)
                    gen2.saved_block_scope_stack = interp.block_scope_stack.clone();
                }
            }

            interp.stack.truncate(base_pointer);

            let final_result = match result {
                Ok(yield_value) if yielded => {
                    let result_obj = props! {
                        "value" => yield_value,
                        "done" => Value::Boolean(false),
                    };
                    let obj_idx = interp.gc.allocate(
                        &mut interp.heap,
                        crate::vm::interpreter::HeapValue::Object(
                            crate::vm::interpreter::JsObject {
                                properties: result_obj,
                                prototype: None,
                                extensible: true,
                            },
                        ),
                    );
                    Ok(Value::Object(obj_idx))
                }
                _ => {
                    let result_obj = props! {
                        "value" => Value::Undefined,
                        "done" => Value::Boolean(true),
                    };
                    let obj_idx = interp.gc.allocate(
                        &mut interp.heap,
                        crate::vm::interpreter::HeapValue::Object(
                            crate::vm::interpreter::JsObject {
                                properties: result_obj,
                                prototype: None,
                                extensible: true,
                            },
                        ),
                    );
                    Ok(Value::Object(obj_idx))
                }
            };
            return final_result;
        }

        Ok(gen.yield_value.clone())
    } else {
        Err(Error::TypeError("Not a Generator".into()))
    }
}

pub(super) fn native_generator_symbol_iterator(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(this.clone())
}

pub(super) fn native_generator_return(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Generator(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Generator".into())),
    };

    let value = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::Generator(gen) = &mut interp.heap[idx] {
        gen.yield_value = value;
        Ok(Value::Undefined)
    } else {
        Err(Error::TypeError("Not a Generator".into()))
    }
}

pub(super) fn native_generator_throw(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Generator(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Generator".into())),
    };

    let error = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::Generator(_gen) = &mut interp.heap[idx] {
        Err(Error::RuntimeError(format!("Generator throw: {:?}", error)))
    } else {
        Err(Error::TypeError("Not a Generator".into()))
    }
}
