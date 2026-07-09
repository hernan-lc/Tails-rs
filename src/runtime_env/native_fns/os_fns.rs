use crate::errors::Result;
use crate::objects::Value;
use crate::props;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

pub(super) fn native_os_platform(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::from_string(tails_os::platform().to_string()))
}

pub(super) fn native_os_arch(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::from_string(tails_os::arch().to_string()))
}

pub(super) fn native_os_cpus(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let cpus_info = tails_os::cpus();
    let cpus: Vec<Value> = cpus_info
        .iter()
        .map(|cpu| {
            let times_props = props! {
                "user" => Value::Integer(cpu.times.user),
                "nice" => Value::Integer(cpu.times.nice),
                "sys" => Value::Integer(cpu.times.sys),
                "idle" => Value::Integer(cpu.times.idle),
                "irq" => Value::Integer(cpu.times.irq),
            };
            let times_idx = interp.gc.allocate(
                &mut interp.heap,
                HeapValue::Object(JsObject {
                    properties: times_props,
                    prototype: None,
                    extensible: true,
                }),
            );
            let props = props! {
                "model" => Value::from_string(cpu.model.clone()),
                "speed" => Value::Float(cpu.speed),
                "times" => Value::Object(times_idx),
            };
            let cpu_idx = interp.heap.len();
            interp.heap.push(HeapValue::Object(JsObject {
                properties: props,
                prototype: None,
                extensible: true,
            }));
            Value::Object(cpu_idx)
        })
        .collect();

    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: cpus,
        }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_os_totalmem(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Float(tails_os::totalmem()))
}

pub(super) fn native_os_freemem(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Float(tails_os::freemem()))
}

pub(super) fn native_os_uptime(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Float(tails_os::uptime()))
}

pub(super) fn native_os_hostname(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    match tails_os::hostname() {
        Ok(h) => Ok(Value::from_string(h)),
        Err(_) => Ok(Value::from_string("localhost".to_string())),
    }
}

pub(super) fn native_os_type(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::from_string(tails_os::os_type().to_string()))
}

pub(super) fn native_os_release(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::from_string(tails_os::release()))
}

pub(super) fn native_os_homedir(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::from_string(tails_os::homedir()))
}

pub(super) fn native_os_tmpdir(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::from_string(tails_os::tmpdir()))
}
