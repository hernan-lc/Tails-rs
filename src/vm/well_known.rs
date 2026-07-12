//! Centralized well-known JavaScript string constants.
//!
//! Never compare against or build these literals inline — use the constants
//! below. This mirrors the existing `crate::runtime_env::native_fns::constants`
//! module (native-function indices) but for string values, and keeps the
//! runtime free of magic strings that could be mistyped or drift apart.

// ---------------------------------------------------------------------------
// A. Primitive literal representations
// ---------------------------------------------------------------------------
pub const UNDEFINED: &str = "undefined";
pub const NULL: &str = "null";
pub const NAN: &str = "NaN";
pub const INFINITY: &str = "Infinity";
pub const TRUE: &str = "true";
pub const FALSE: &str = "false";

// ---------------------------------------------------------------------------
// B. Well-known property names
// ---------------------------------------------------------------------------
pub const PROTOTYPE: &str = "prototype";
pub const CONSTRUCTOR: &str = "constructor";
pub const LENGTH: &str = "length";
pub const NAME: &str = "name";
pub const MESSAGE: &str = "message";
pub const STACK: &str = "stack";
pub const CAUSE: &str = "cause";
pub const TO_STRING: &str = "toString";
pub const VALUE_OF: &str = "valueOf";
pub const TO_LOCALE_STRING: &str = "toLocaleString";
pub const THEN: &str = "then";
pub const CATCH: &str = "catch";
pub const FINALLY: &str = "finally";
pub const SYMBOL_ITERATOR: &str = "Symbol.iterator";
pub const SYMBOL_TO_STRING_TAG: &str = "Symbol.toStringTag";

// Collection / object methods (lowercase JS names, not constructors).
pub const GET: &str = "get";
pub const SET_PROP: &str = "set"; // method name; `SET` is the Set constructor
pub const HAS: &str = "has";
pub const DELETE: &str = "delete";
pub const ADD: &str = "add";
pub const CLEAR: &str = "clear";
pub const SIZE: &str = "size";
pub const KEYS: &str = "keys";
pub const VALUES: &str = "values";
pub const ENTRIES: &str = "entries";
pub const FOR_EACH: &str = "forEach";

// Iterator protocol
pub const NEXT: &str = "next";
pub const DONE: &str = "done";
pub const VALUE: &str = "value";
pub const RETURN: &str = "return";

// TypedArray / Buffer
pub const BYTE_LENGTH: &str = "byteLength";
pub const BYTE_OFFSET: &str = "byteOffset";
/// Property name on TypedArray / module id for the Buffer builtin (same spelling).
pub const BUFFER: &str = "buffer";

// Proxy traps (aliases of GET / SET_PROP / HAS where the spellings match)
pub const TRAP_GET: &str = GET;
pub const TRAP_SET: &str = SET_PROP;
pub const TRAP_HAS: &str = HAS;
pub const TRAP_DELETE_PROPERTY: &str = "deleteProperty";
pub const TRAP_APPLY: &str = "apply";
pub const TRAP_CONSTRUCT: &str = "construct";

// Internal storage prefixes (object accessors / methods)
pub const GETTER_PREFIX: &str = "__getter_";
pub const SETTER_PREFIX: &str = "__setter_";
pub const METHOD_PREFIX: &str = "__method_";

// ---------------------------------------------------------------------------
// C. Error type names (mirror `ErrorKind` in crate::errors)
// ---------------------------------------------------------------------------
pub const TYPE_ERROR: &str = "TypeError";
pub const REFERENCE_ERROR: &str = "ReferenceError";
pub const SYNTAX_ERROR: &str = "SyntaxError";
pub const RANGE_ERROR: &str = "RangeError";
pub const URI_ERROR: &str = "URIError";
pub const EVAL_ERROR: &str = "EvalError";
pub const AGGREGATE_ERROR: &str = "AggregateError";
pub const PARSE_ERROR: &str = "ParseError";
pub const RUNTIME_ERROR: &str = "RuntimeError";
pub const INTERNAL_ERROR: &str = "InternalError";

// ---------------------------------------------------------------------------
// D. Global object / builtin names
// ---------------------------------------------------------------------------
pub const GLOBAL_THIS: &str = "globalThis";
pub const OBJECT: &str = "Object";
pub const ARRAY: &str = "Array";
pub const FUNCTION: &str = "Function";
pub const STRING: &str = "String";
pub const NUMBER: &str = "Number";
pub const BOOLEAN: &str = "Boolean";
pub const SYMBOL: &str = "Symbol";
pub const MATH: &str = "Math";
pub const JSON: &str = "JSON";
pub const PROMISE: &str = "Promise";
pub const ERROR: &str = "Error";
pub const REGEXP: &str = "RegExp";
pub const DATE: &str = "Date";
pub const BIGINT: &str = "BigInt";
pub const MAP: &str = "Map";
pub const SET: &str = "Set";
pub const PROXY: &str = "Proxy";
pub const REFLECT: &str = "Reflect";
pub const CONSOLE: &str = "console";
pub const PROCESS: &str = "process";

// ---------------------------------------------------------------------------
// E. Module specifiers
// ---------------------------------------------------------------------------
pub const MOD_FS: &str = "fs";
pub const MOD_FS_PROMISES: &str = "fs/promises";
pub const MOD_PATH: &str = "path";
pub const MOD_PROCESS: &str = PROCESS;
pub const MOD_OS: &str = "os";
pub const MOD_HTTP: &str = "http";
pub const MOD_NET: &str = "net";
pub const MOD_WEBSOCKET: &str = "websocket";
pub const MOD_CRYPTO: &str = "crypto";
pub const MOD_DNS: &str = "dns";
pub const MOD_URL: &str = "url";
pub const MOD_UTIL: &str = "util";
pub const MOD_EVENTS: &str = "events";
pub const MOD_BUFFER: &str = BUFFER;
pub const MOD_STREAM: &str = "stream";
pub const MOD_QUERYSTRING: &str = "querystring";
pub const MOD_ZLIB: &str = "zlib";
pub const MOD_TLS: &str = "tls";
pub const MOD_TIMERS: &str = "timers";
pub const MOD_ASSERT: &str = "assert";
pub const MOD_PERF_HOOKS: &str = "perf_hooks";
pub const MOD_ASYNC_HOOKS: &str = "async_hooks";
pub const MOD_WORKER_THREADS: &str = "worker_threads";

// ---------------------------------------------------------------------------
// F. `typeof` result tags
// ---------------------------------------------------------------------------
pub const TYPEOF_UNDEFINED: &str = UNDEFINED;
pub const TYPEOF_OBJECT: &str = "object";
pub const TYPEOF_BOOLEAN: &str = "boolean";
pub const TYPEOF_NUMBER: &str = "number";
pub const TYPEOF_STRING: &str = "string";
pub const TYPEOF_BIGINT: &str = "bigint";
pub const TYPEOF_SYMBOL: &str = "symbol";
pub const TYPEOF_FUNCTION: &str = "function";

// ---------------------------------------------------------------------------
// G. Runtime limits / tuning (shared numeric constants)
// ---------------------------------------------------------------------------
/// Default max call-stack depth for the interpreter.
pub const DEFAULT_MAX_CALL_STACK_DEPTH: usize = 10_000;
/// Cap on cached `eval` compilations (source hash → bytecode).
pub const EVAL_CACHE_MAX: usize = 128;
/// Loop back-edge hits before the baseline JIT compiles a hot loop.
pub const JIT_DEFAULT_THRESHOLD: u32 = 100;
/// Pre-sized microtask queue capacity.
pub const MICROTASK_QUEUE_INITIAL_CAP: usize = 64;

/// Collection method names that Map/Set/WeakMap/WeakSet share or specialize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionMethod {
    Get,
    Set,
    Has,
    Delete,
    Add,
    Clear,
    Size,
    Keys,
    Values,
    Entries,
    ForEach,
}

impl CollectionMethod {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => GET,
            Self::Set => SET_PROP,
            Self::Has => HAS,
            Self::Delete => DELETE,
            Self::Add => ADD,
            Self::Clear => CLEAR,
            Self::Size => SIZE,
            Self::Keys => KEYS,
            Self::Values => VALUES,
            Self::Entries => ENTRIES,
            Self::ForEach => FOR_EACH,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn no_duplicate_values() {
        let all = [
            UNDEFINED,
            NULL,
            NAN,
            INFINITY,
            TRUE,
            FALSE,
            PROTOTYPE,
            CONSTRUCTOR,
            LENGTH,
            NAME,
            MESSAGE,
            STACK,
            CAUSE,
            TO_STRING,
            VALUE_OF,
            TO_LOCALE_STRING,
            THEN,
            CATCH,
            FINALLY,
            SYMBOL_ITERATOR,
            SYMBOL_TO_STRING_TAG,
            GET,
            SET_PROP,
            HAS,
            DELETE,
            ADD,
            CLEAR,
            SIZE,
            KEYS,
            VALUES,
            ENTRIES,
            FOR_EACH,
            NEXT,
            DONE,
            VALUE,
            RETURN,
            BYTE_LENGTH,
            BYTE_OFFSET,
            BUFFER,
            // TRAP_GET/SET/HAS and MOD_BUFFER alias existing strings — omitted
            TRAP_DELETE_PROPERTY,
            TRAP_APPLY,
            TRAP_CONSTRUCT,
            GETTER_PREFIX,
            SETTER_PREFIX,
            METHOD_PREFIX,
            TYPE_ERROR,
            REFERENCE_ERROR,
            SYNTAX_ERROR,
            RANGE_ERROR,
            URI_ERROR,
            EVAL_ERROR,
            AGGREGATE_ERROR,
            PARSE_ERROR,
            RUNTIME_ERROR,
            INTERNAL_ERROR,
            GLOBAL_THIS,
            OBJECT,
            ARRAY,
            FUNCTION,
            STRING,
            NUMBER,
            BOOLEAN,
            SYMBOL,
            MATH,
            JSON,
            PROMISE,
            ERROR,
            REGEXP,
            DATE,
            BIGINT,
            MAP,
            SET,
            PROXY,
            REFLECT,
            CONSOLE,
            PROCESS,
            MOD_FS,
            MOD_FS_PROMISES,
            MOD_PATH,
            // MOD_PROCESS aliases PROCESS
            MOD_OS,
            MOD_HTTP,
            MOD_NET,
            MOD_WEBSOCKET,
            MOD_CRYPTO,
            MOD_DNS,
            MOD_URL,
            MOD_UTIL,
            MOD_EVENTS,
            MOD_STREAM,
            MOD_QUERYSTRING,
            MOD_ZLIB,
            MOD_TLS,
            MOD_TIMERS,
            MOD_ASSERT,
            TYPEOF_OBJECT,
            TYPEOF_BOOLEAN,
            TYPEOF_NUMBER,
            TYPEOF_STRING,
            TYPEOF_BIGINT,
            TYPEOF_SYMBOL,
            TYPEOF_FUNCTION,
        ];
        let mut seen = HashSet::new();
        for s in all {
            // TRAP_GET/SET/HAS intentionally equal GET/SET_PROP/HAS — skip aliases
            // by only inserting primary names above.
            assert!(seen.insert(s), "duplicate well-known string: {:?}", s);
        }
    }

    #[test]
    fn collection_method_strs_match_consts() {
        assert_eq!(CollectionMethod::Get.as_str(), GET);
        assert_eq!(CollectionMethod::Set.as_str(), SET_PROP);
        assert_eq!(CollectionMethod::Has.as_str(), HAS);
        assert_eq!(CollectionMethod::Delete.as_str(), DELETE);
        assert_eq!(CollectionMethod::Add.as_str(), ADD);
    }
}
