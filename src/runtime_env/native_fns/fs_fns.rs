use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::props;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};
use rustc_hash::FxHashMap;

use super::helpers::to_string_value;

pub(super) fn native_fs_read_file_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let content = tails_fs::read_file(&path).map_err(|_| {
        Error::RuntimeError(format!(
            "ENOENT: no such file or directory, open '{}'",
            path
        ))
    })?;
    Ok(Value::from_string(content))
}

pub(super) fn native_fs_write_file_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let content = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    tails_fs::write_file(&path, &content)
        .map_err(|_| Error::RuntimeError(format!("EACCES: permission denied, open '{}'", path)))?;
    Ok(Value::Undefined)
}

pub(super) fn native_fs_exists_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    Ok(Value::Boolean(tails_fs::exists(&path)))
}

pub(super) fn native_fs_mkdir_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let recursive = args
        .get(1)
        .and_then(|v| {
            if let Value::Object(idx) = v {
                if let HeapValue::Object(obj) = &interp.heap[*idx] {
                    return obj
                        .properties
                        .get("recursive")
                        .map(|v| matches!(v, Value::Boolean(true)));
                }
            }
            Some(false)
        })
        .unwrap_or(false);
    tails_fs::mkdir(&path, recursive)
        .map_err(|_| Error::RuntimeError(format!("EACCES: permission denied, mkdir '{}'", path)))?;
    Ok(Value::Undefined)
}

pub(super) fn native_fs_readdir_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let entries: Vec<Value> = tails_fs::readdir(&path)
        .map_err(|_| {
            Error::RuntimeError(format!(
                "ENOENT: no such file or directory, scandir '{}'",
                path
            ))
        })?
        .into_iter()
        .map(Value::from_string)
        .collect();
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: entries,
        }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_fs_stat_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let stat = tails_fs::stat(&path).map_err(|_| {
        Error::RuntimeError(format!(
            "ENOENT: no such file or directory, stat '{}'",
            path
        ))
    })?;
    // Static baseline of the stat result. The platform-specific
    // `mode`, `mtimeMs`, and `birthtimeMs` are merged in below when
    // the underlying `tails_fs::stat` call reported them.
    let mut props = props! {
        "size" => Value::Integer(stat.size as i64),
        "isFile" => Value::Boolean(stat.is_file),
        "isDirectory" => Value::Boolean(stat.is_directory),
        "isSymbolicLink" => Value::Boolean(stat.is_symbolic_link),
    };
    if let Some(mode) = stat.mode {
        props.insert("mode".into(), Value::Integer(mode as i64));
    }
    if let Some(mtime) = stat.mtime_ms {
        props.insert("mtimeMs".into(), Value::Float(mtime));
    }
    if let Some(birthtime) = stat.birthtime_ms {
        props.insert("birthtimeMs".into(), Value::Float(birthtime));
    }
    let stat_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));
    Ok(Value::Object(stat_idx))
}

pub(super) fn native_fs_unlink_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    tails_fs::unlink(&path).map_err(|_| {
        Error::RuntimeError(format!(
            "ENOENT: no such file or directory, unlink '{}'",
            path
        ))
    })?;
    Ok(Value::Undefined)
}

pub(super) fn native_fs_rm_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let recursive = args
        .get(1)
        .and_then(|v| {
            if let Value::Object(idx) = v {
                if let HeapValue::Object(obj) = &interp.heap[*idx] {
                    return obj
                        .properties
                        .get("recursive")
                        .map(|v| matches!(v, Value::Boolean(true)));
                }
            }
            Some(false)
        })
        .unwrap_or(false);
    tails_fs::rm(&path, recursive).map_err(|_| {
        Error::RuntimeError(format!("ENOENT: no such file or directory, rm '{}'", path))
    })?;
    Ok(Value::Undefined)
}

pub(super) fn native_fs_copy_file_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let src = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let dest = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    tails_fs::copy_file(&src, &dest).map_err(|_| {
        Error::RuntimeError(format!(
            "EACCES: permission denied, copy '{}' to '{}'",
            src, dest
        ))
    })?;
    Ok(Value::Undefined)
}

pub(super) fn native_fs_rename_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let old_path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let new_path = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    tails_fs::rename(&old_path, &new_path).map_err(|_| {
        Error::RuntimeError(format!(
            "EACCES: permission denied, rename '{}' to '{}'",
            old_path, new_path
        ))
    })?;
    Ok(Value::Undefined)
}

pub(super) fn native_fs_append_file_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let content = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    tails_fs::append_file(&path, &content)
        .map_err(|_| Error::RuntimeError(format!("EACCES: permission denied, open '{}'", path)))?;
    Ok(Value::Undefined)
}

pub(super) fn native_fs_readdir(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    match tails_fs::readdir(&path) {
        Ok(names) => {
            let entries: Vec<Value> = names.into_iter().map(Value::from_string).collect();
            let arr_idx = interp.heap.len();
            interp
                .heap
                .push(HeapValue::Array(crate::vm::interpreter::JsArray {
                    elements: entries,
                }));
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::fulfilled(Value::Array(arr_idx)),
            ));
            Ok(Value::Promise(promise_idx))
        }
        Err(e) => {
            let err_idx = interp.heap.len();
            let props = props! {
                "message" => Value::from_string(format!(
                    "ENOENT: no such file or directory, scandir '{}': {}",
                    path, e
                )),
            };
            interp
                .heap
                .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                    properties: props,
                    prototype: None,
                    extensible: true,
                }));
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::rejected(Value::Object(err_idx)),
            ));
            Ok(Value::Promise(promise_idx))
        }
    }
}

pub(super) fn native_fs_read_file(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    match tails_fs::read_file(&path) {
        Ok(content) => {
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::fulfilled(Value::from_string(
                    content,
                )),
            ));
            Ok(Value::Promise(promise_idx))
        }
        Err(e) => {
            let err_idx = interp.heap.len();
            let props = props! {
                "message" => Value::from_string(format!(
                    "ENOENT: no such file or directory, open '{}': {}",
                    path, e
                )),
            };
            interp
                .heap
                .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                    properties: props,
                    prototype: None,
                    extensible: true,
                }));
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::rejected(Value::Object(err_idx)),
            ));
            Ok(Value::Promise(promise_idx))
        }
    }
}

pub(super) fn native_fs_write_file(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let content = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    match tails_fs::write_file(&path, &content) {
        Ok(()) => {
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::fulfilled(Value::Undefined),
            ));
            Ok(Value::Promise(promise_idx))
        }
        Err(e) => {
            let err_idx = interp.heap.len();
            let props = props! {
                "message" => Value::from_string(format!("EACCES: permission denied, open '{}': {}", path, e)),
            };
            interp
                .heap
                .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                    properties: props,
                    prototype: None,
                    extensible: true,
                }));
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::rejected(Value::Object(err_idx)),
            ));
            Ok(Value::Promise(promise_idx))
        }
    }
}

pub(super) fn native_fs_stat(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    match tails_fs::stat(&path) {
        Ok(stat) => {
            let mut props = FxHashMap::default();
            props.insert("size".into(), Value::Integer(stat.size as i64));
            props.insert("isFile".into(), Value::Boolean(stat.is_file));
            props.insert("isDirectory".into(), Value::Boolean(stat.is_directory));
            props.insert(
                "isSymbolicLink".into(),
                Value::Boolean(stat.is_symbolic_link),
            );
            if let Some(mode) = stat.mode {
                props.insert("mode".into(), Value::Integer(mode as i64));
            }
            if let Some(mtime) = stat.mtime_ms {
                props.insert("mtimeMs".into(), Value::Float(mtime));
            }
            let stat_idx = interp.heap.len();
            interp
                .heap
                .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                    properties: props.into(),
                    prototype: None,
                    extensible: true,
                }));
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::fulfilled(Value::Object(stat_idx)),
            ));
            Ok(Value::Promise(promise_idx))
        }
        Err(e) => {
            let err_idx = interp.heap.len();
            let props = props! {
                "message" => Value::from_string(format!(
                    "ENOENT: no such file or directory, stat '{}': {}",
                    path, e
                )),
            };
            interp
                .heap
                .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                    properties: props,
                    prototype: None,
                    extensible: true,
                }));
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::rejected(Value::Object(err_idx)),
            ));
            Ok(Value::Promise(promise_idx))
        }
    }
}

pub(super) fn native_fs_mkdir(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let recursive = args
        .get(1)
        .and_then(|v| {
            if let Value::Object(idx) = v {
                if let HeapValue::Object(obj) = &interp.heap[*idx] {
                    return obj
                        .properties
                        .get("recursive")
                        .map(|v| matches!(v, Value::Boolean(true)));
                }
            }
            Some(false)
        })
        .unwrap_or(false);

    match tails_fs::mkdir(&path, recursive) {
        Ok(()) => {
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::fulfilled(Value::Undefined),
            ));
            Ok(Value::Promise(promise_idx))
        }
        Err(e) => {
            let err_idx = interp.heap.len();
            let props = props! {
                "message" => Value::from_string(format!(
                    "EACCES: permission denied, mkdir '{}': {}",
                    path, e
                )),
            };
            interp
                .heap
                .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                    properties: props,
                    prototype: None,
                    extensible: true,
                }));
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::rejected(Value::Object(err_idx)),
            ));
            Ok(Value::Promise(promise_idx))
        }
    }
}

pub(super) fn native_fs_unlink(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    match tails_fs::unlink(&path) {
        Ok(()) => {
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::fulfilled(Value::Undefined),
            ));
            Ok(Value::Promise(promise_idx))
        }
        Err(e) => {
            let err_idx = interp.heap.len();
            let props = props! {
                "message" => Value::from_string(format!(
                    "ENOENT: no such file or directory, unlink '{}': {}",
                    path, e
                )),
            };
            interp
                .heap
                .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                    properties: props,
                    prototype: None,
                    extensible: true,
                }));
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::rejected(Value::Object(err_idx)),
            ));
            Ok(Value::Promise(promise_idx))
        }
    }
}

pub(super) fn native_fs_copy_file(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let src = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let dest = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    match tails_fs::copy_file(&src, &dest) {
        Ok(_) => {
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::fulfilled(Value::Undefined),
            ));
            Ok(Value::Promise(promise_idx))
        }
        Err(e) => {
            let err_idx = interp.heap.len();
            let props = props! {
                "message" => Value::from_string(format!(
                    "EACCES: permission denied, copy '{}' to '{}': {}",
                    src, dest, e
                )),
            };
            interp
                .heap
                .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                    properties: props,
                    prototype: None,
                    extensible: true,
                }));
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::rejected(Value::Object(err_idx)),
            ));
            Ok(Value::Promise(promise_idx))
        }
    }
}

pub(super) fn native_fs_rename(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let old_path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let new_path = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    match tails_fs::rename(&old_path, &new_path) {
        Ok(()) => {
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::fulfilled(Value::Undefined),
            ));
            Ok(Value::Promise(promise_idx))
        }
        Err(e) => {
            let err_idx = interp.heap.len();
            let props = props! {
                "message" => Value::from_string(format!(
                    "EACCES: permission denied, rename '{}' to '{}': {}",
                    old_path, new_path, e
                )),
            };
            interp
                .heap
                .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                    properties: props,
                    prototype: None,
                    extensible: true,
                }));
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(
                crate::objects::js_promise::JsPromise::rejected(Value::Object(err_idx)),
            ));
            Ok(Value::Promise(promise_idx))
        }
    }
}
