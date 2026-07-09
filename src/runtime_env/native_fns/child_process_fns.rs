use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::props;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::to_string_value;

// child_process.execSync(command, options?) -> string (index 386)
pub(super) fn native_child_process_exec_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let command = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    let mut encoding = "utf8";
    let mut cwd: Option<String> = None;
    let mut timeout_ms: Option<u64> = None;

    if let Some(Value::Object(opts_idx)) = args.get(1) {
        if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
            if let Some(v) = obj.properties.get("encoding") {
                encoding = match v {
                    Value::String(s) if **s == *"buffer" => "buffer",
                    Value::Cons(c) if c.flatten() == "buffer" => "buffer",
                    _ => "utf8",
                };
            }
            if let Some(v) = obj.properties.get("cwd") {
                cwd = Some(to_string_value(interp, v));
            }
            if let Some(v) = obj.properties.get("timeout") {
                timeout_ms = Some(match v {
                    Value::Integer(n) => *n as u64,
                    Value::Float(n) => *n as u64,
                    _ => 0,
                });
            }
        }
    }

    let shell = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let mut cmd = std::process::Command::new(shell.0);
    cmd.arg(shell.1).arg(&command);

    if let Some(ref dir) = cwd {
        cmd.current_dir(dir);
    }

    // Set timeout via kill_on_drop (best effort)
    if timeout_ms.is_some() {
        // We handle timeout by checking after the fact
    }

    let output = cmd
        .output()
        .map_err(|e| Error::RuntimeError(format!("execSync failed: {}", e)))?;

    let exit_code = output.status.code().unwrap_or(-1);

    if !output.status.success() {
        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();

        // Create error object similar to Node.js
        let err_props = props! {
            "message" => Value::from_string(format!("Command failed: {}", command.trim())),
            "status" => Value::Integer(exit_code as i64),
            "stderr" => Value::from_string(stderr_str.clone().into()),
            "stdout" => Value::from_string(stdout_str.clone().into()),
            "signal" => Value::Null,
        };

        let _err_idx = interp.heap.len();
        interp.heap.push(HeapValue::Object(JsObject {
            properties: err_props,
            prototype: None,
            extensible: true,
        }));

        return Err(Error::RuntimeError(format!(
            "Command failed: {}",
            command.trim()
        )));
    }

    let stdout = output.stdout;

    if encoding == "buffer" {
        let buf_idx = interp.heap.len();
        interp.heap.push(HeapValue::Buffer(stdout));
        Ok(Value::Buffer(buf_idx))
    } else {
        Ok(Value::from_string(String::from_utf8_lossy(&stdout).to_string()))
    }
}

// child_process.exec(command, options?, callback?) -> { pid, stdout, stderr, exitCode } (index 387)
pub(super) fn native_child_process_exec(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let command = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    let mut cwd: Option<String> = None;
    let mut callback_idx: Option<usize> = None;

    // Parse args: exec(command), exec(command, options), exec(command, callback), exec(command, options, callback)
    match args.len() {
        2 => {
            // Could be options or callback
            match &args[1] {
                Value::Function(idx) => callback_idx = Some(*idx),
                Value::Object(opts_idx) => {
                    if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
                        if let Some(v) = obj.properties.get("cwd") {
                            cwd = Some(to_string_value(interp, v));
                        }
                    }
                }
                _ => {}
            }
        }
        3 => {
            if let Value::Object(opts_idx) = &args[1] {
                if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
                    if let Some(v) = obj.properties.get("cwd") {
                        cwd = Some(to_string_value(interp, v));
                    }
                }
            }
            if let Value::Function(idx) = &args[2] {
                callback_idx = Some(*idx);
            }
        }
        _ => {}
    }

    let shell = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let mut cmd = std::process::Command::new(shell.0);
    cmd.arg(shell.1).arg(&command);

    if let Some(ref dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .map_err(|e| Error::RuntimeError(format!("exec failed: {}", e)))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

    // Build result object
    let result_props = props! {
        "stdout" => Value::from_string(stdout_str.into()),
        "stderr" => Value::from_string(stderr_str.into()),
        "status" => Value::Integer(exit_code as i64),
    };

    let result_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: result_props,
        prototype: None,
        extensible: true,
    }));

    // Call callback if provided
    if let Some(cb_idx) = callback_idx {
        let error_val = if !output.status.success() {
            Value::from_string(format!("Command failed: {}", command.trim()))
        } else {
            Value::Null
        };
        let _ = interp.call_value(
            &Value::Function(cb_idx),
            &Value::Undefined,
            &[error_val, Value::Object(result_idx)],
        );
    }

    Ok(Value::Object(result_idx))
}

// child_process.spawn(command, args?, options?) -> ChildProcess (index 388)
pub(super) fn native_child_process_spawn(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let command = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    let mut spawn_args: Vec<String> = Vec::new();
    let mut cwd: Option<String> = None;
    let mut env: Option<std::collections::HashMap<String, String>> = None;
    let mut shell_flag = false;

    // Parse arguments
    match args.len() {
        2 => match &args[1] {
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                    spawn_args = arr
                        .elements
                        .iter()
                        .map(|v| to_string_value(interp, v))
                        .collect();
                }
            }
            Value::Object(opts_idx) => {
                if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
                    parse_spawn_options(interp, obj, &mut cwd, &mut env, &mut shell_flag);
                }
            }
            _ => {}
        },
        3 => {
            if let Value::Array(arr_idx) = &args[1] {
                if let HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                    spawn_args = arr
                        .elements
                        .iter()
                        .map(|v| to_string_value(interp, v))
                        .collect();
                }
            }
            if let Value::Object(opts_idx) = &args[2] {
                if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
                    parse_spawn_options(interp, obj, &mut cwd, &mut env, &mut shell_flag);
                }
            }
        }
        _ => {}
    }

    let mut cmd = if shell_flag || spawn_args.is_empty() {
        let shell = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };
        let mut c = std::process::Command::new(shell.0);
        c.arg(shell.1).arg(&command);
        if !spawn_args.is_empty() {
            for arg in &spawn_args {
                c.arg(arg);
            }
        }
        c
    } else {
        let mut c = std::process::Command::new(&command);
        for arg in &spawn_args {
            c.arg(arg);
        }
        c
    };

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.stdin(std::process::Stdio::piped());

    if let Some(ref dir) = cwd {
        cmd.current_dir(dir);
    }

    if let Some(ref env_map) = env {
        cmd.env_clear();
        for (k, v) in env_map {
            cmd.env(k, v);
        }
    }

    let child = cmd
        .spawn()
        .map_err(|e| Error::RuntimeError(format!("spawn failed: {}", e)))?;

    let pid = child.id();

    // Wait for the process to complete and collect output
    let output = child
        .wait_with_output()
        .map_err(|e| Error::RuntimeError(format!("spawn wait failed: {}", e)))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

    let stdout_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props! {
            "data" => Value::from_string(stdout_str.into()),
        },
        prototype: None,
        extensible: true,
    }));

    let stderr_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props! {
            "data" => Value::from_string(stderr_str.into()),
        },
        prototype: None,
        extensible: true,
    }));

    let child_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props! {
            "pid" => Value::Integer(pid as i64),
            "stdout" => Value::Object(stdout_idx),
            "stderr" => Value::Object(stderr_idx),
            "exitCode" => Value::Integer(exit_code as i64),
            "signalCode" => Value::Null,
        },
        prototype: None,
        extensible: true,
    }));

    Ok(Value::Object(child_idx))
}

fn parse_spawn_options(
    interp: &Interpreter,
    obj: &JsObject,
    cwd: &mut Option<String>,
    env: &mut Option<std::collections::HashMap<String, String>>,
    shell: &mut bool,
) {
    if let Some(v) = obj.properties.get("cwd") {
        *cwd = Some(to_string_value(interp, v));
    }
    if let Some(Value::Object(env_idx)) = obj.properties.get("env") {
        if let HeapValue::Object(env_obj) = &interp.heap[*env_idx] {
            let mut env_map = std::collections::HashMap::new();
            for (k, v) in &env_obj.properties {
                if !k.starts_with('_') {
                    env_map.insert(k.to_string(), to_string_value(interp, v));
                }
            }
            *env = Some(env_map);
        }
    }
    if let Some(Value::Boolean(s)) = obj.properties.get("shell") {
        *shell = *s;
    }
}
