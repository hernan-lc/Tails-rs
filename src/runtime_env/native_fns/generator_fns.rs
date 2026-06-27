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

    // First, get the generator info without mutating
    let (func_heap_idx, resume_pc) =
        if let crate::vm::interpreter::HeapValue::Generator(gen) = &interp.heap[idx] {
            (gen.func_heap_idx, gen.resume_pc)
        } else {
            return Err(Error::TypeError("Not a Generator".into()));
        };

    // Now mutate to push the value and save stack
    if let crate::vm::interpreter::HeapValue::Generator(gen) = &mut interp.heap[idx] {
        // Push the value to the stack so it becomes the result of the yield expression
        interp.stack.push(value);

        // Restore the saved stack
        if !gen.saved_stack.is_empty() {
            interp.stack.append(&mut gen.saved_stack);
        }

        // Execute from the resume PC using execute_from
        let module = interp.current_module.clone();
        if let Some(module) = module {
            // Set up call frame for generator execution
            let return_address = module.instructions.len();
            let base_pointer = interp.stack.len();
            let closure_count = 0;

            interp.call_stack.push(crate::vm::interpreter::CallFrame {
                return_address,
                base_pointer,
                closure_var_count: closure_count,
                func_heap_idx,
                this_value: None,
                is_construct: false,
            });

            // Execute from the resume PC using the module's execute_from
            let result = interp.execute_from(&module, resume_pc);

            // Save the current stack state back to generator
            if let Some(frame) = interp.call_stack.last() {
                if let crate::vm::interpreter::HeapValue::Generator(gen2) = &mut interp.heap[idx] {
                    gen2.saved_stack = interp.stack[frame.base_pointer..].to_vec();
                    gen2.resume_pc = frame.return_address;
                }
            }

            return result;
        }

        // Return the yield value
        if let crate::vm::interpreter::HeapValue::Generator(gen) = &interp.heap[idx] {
            Ok(gen.yield_value.clone())
        } else {
            Ok(Value::Undefined)
        }
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
        // For now, just throw the error
        Err(Error::RuntimeError(format!("Generator throw: {:?}", error)))
    } else {
        Err(Error::TypeError("Not a Generator".into()))
    }
}
