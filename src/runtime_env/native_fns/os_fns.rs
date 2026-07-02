use rustc_hash::FxHashMap;
use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

pub(super) fn native_os_platform(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::String(tails_os::platform().to_string()))
}

pub(super) fn native_os_arch(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::String(tails_os::arch().to_string()))
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
            let mut times_props = FxHashMap::default();
            times_props.insert("user".into(), Value::Integer(cpu.times.user));
            times_props.insert("nice".into(), Value::Integer(cpu.times.nice));
            times_props.insert("sys".into(), Value::Integer(cpu.times.sys));
            times_props.insert("idle".into(), Value::Integer(cpu.times.idle));
            times_props.insert("irq".into(), Value::Integer(cpu.times.irq));
            let times_idx = interp.gc.allocate(
                &mut interp.heap,
                HeapValue::Object(JsObject {
                    properties: times_props,
                    prototype: None,
                    extensible: true,
                }),
            );
            let mut props = FxHashMap::default();
            props.insert("model".into(), Value::String(cpu.model.clone()));
            props.insert("speed".into(), Value::Float(cpu.speed));
            props.insert("times".into(), Value::Object(times_idx));
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
        Ok(h) => Ok(Value::String(h)),
        Err(_) => Ok(Value::String("localhost".to_string())),
    }
}

pub(super) fn native_os_type(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::String(tails_os::os_type().to_string()))
}

pub(super) fn native_os_release(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::String(tails_os::release()))
}

pub(super) fn native_os_homedir(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::String(tails_os::homedir()))
}

pub(super) fn native_os_tmpdir(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::String(tails_os::tmpdir()))
}
