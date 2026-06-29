use crate::errors::{Error, Result};
use crate::objects::Value;
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

            // Push saved locals first so they land at base_pointer + 0, base_pointer + 1, etc.
            let saved = gen.saved_stack.clone();
            interp.stack.extend_from_slice(&saved);
            // Push the value passed to .next() after the locals
            interp.stack.push(value);

            // Restore the generator's block scope state
            let saved_block_scope = gen.saved_block_scope_stack.clone();
            let outer_block_scope =
                std::mem::replace(&mut interp.block_scope_stack, saved_block_scope);

            let closure_count = 0;
            let call_frame_len_before = interp.call_stack.len();

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
            });

            let result = interp.execute_from(&module, resume_pc);

            // Restore the caller's block scope state
            interp.block_scope_stack = outer_block_scope;

            // Pop the call frame we pushed
            if interp.call_stack.len() > call_frame_len_before {
                interp.call_stack.pop();
            }

            // Save the current stack state back to generator
            if let crate::vm::interpreter::HeapValue::Generator(gen2) = &mut interp.heap[idx] {
                if let Ok(ref _val) = result {
                    // Generator yielded - save state
                    if interp.stack.len() > base_pointer {
                        gen2.saved_stack = interp.stack[base_pointer..].to_vec();
                    } else {
                        gen2.saved_stack = Vec::new();
                    }
                    gen2.saved_block_scope_stack = interp.block_scope_stack.clone();
                } else {
                    // Generator completed or errored
                    gen2.saved_stack = Vec::new();
                    gen2.saved_block_scope_stack = Vec::new();
                    gen2.resume_pc = usize::MAX;
                }
            }

            // Clean up: remove generator locals from the caller's stack
            interp.stack.truncate(base_pointer);

            let final_result = match result {
                Ok(yield_value) => {
                    // Create {value: yield_value, done: false}
                    let mut result_obj = std::collections::HashMap::new();
                    result_obj.insert("value".into(), yield_value);
                    result_obj.insert("done".into(), Value::Boolean(false));
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
                Err(_) => {
                    // Generator threw or completed
                    let mut result_obj = std::collections::HashMap::new();
                    result_obj.insert("value".into(), Value::Undefined);
                    result_obj.insert("done".into(), Value::Boolean(true));
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
