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
pub const MOD_PROCESS: &str = "process";
pub const MOD_OS: &str = "os";
pub const MOD_HTTP: &str = "http";
pub const MOD_NET: &str = "net";
pub const MOD_WEBSOCKET: &str = "websocket";
pub const MOD_CRYPTO: &str = "crypto";
pub const MOD_DNS: &str = "dns";
pub const MOD_URL: &str = "url";
pub const MOD_UTIL: &str = "util";
pub const MOD_EVENTS: &str = "events";
pub const MOD_BUFFER: &str = "buffer";
pub const MOD_STREAM: &str = "stream";
pub const MOD_QUERYSTRING: &str = "querystring";
pub const MOD_ZLIB: &str = "zlib";
pub const MOD_TLS: &str = "tls";
pub const MOD_TIMERS: &str = "timers";
pub const MOD_ASSERT: &str = "assert";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_duplicate_values() {
        let all = [
            UNDEFINED, NULL, NAN, INFINITY, TRUE, FALSE, PROTOTYPE, CONSTRUCTOR,
            LENGTH, NAME, MESSAGE, STACK, CAUSE, TO_STRING, VALUE_OF, TO_LOCALE_STRING,
            THEN, CATCH, FINALLY, SYMBOL_ITERATOR, SYMBOL_TO_STRING_TAG, TYPE_ERROR,
            REFERENCE_ERROR, SYNTAX_ERROR, RANGE_ERROR, URI_ERROR, EVAL_ERROR,
            AGGREGATE_ERROR, PARSE_ERROR, RUNTIME_ERROR, INTERNAL_ERROR, GLOBAL_THIS,
            OBJECT, ARRAY, FUNCTION, STRING, NUMBER, BOOLEAN, SYMBOL, MATH, JSON,
            PROMISE, ERROR, REGEXP, DATE, BIGINT, MAP, SET, PROXY, REFLECT, CONSOLE,
            PROCESS, MOD_FS, MOD_FS_PROMISES, MOD_PATH, MOD_PROCESS, MOD_OS, MOD_HTTP,
            MOD_NET, MOD_WEBSOCKET, MOD_CRYPTO, MOD_DNS, MOD_URL, MOD_UTIL, MOD_EVENTS,
            MOD_BUFFER, MOD_STREAM, MOD_QUERYSTRING, MOD_ZLIB, MOD_TLS, MOD_TIMERS,
            MOD_ASSERT,
        ];
        for w in all.windows(2) {
            assert_ne!(w[0], w[1], "duplicate well-known string: '{}'", w[0]);
        }
    }
}
