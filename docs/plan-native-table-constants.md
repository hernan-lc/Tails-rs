# Plan: Eliminate NATIVE_TABLE Magic Numbers

## Problem

NATIVE_TABLE indices are manually maintained as raw integers across 11+ files (455 occurrences). When adding or reordering functions, the count silently drifts — causing wrong functions to be called (e.g., boolean methods returning NaN because they pointed to `parse_float`).

## Current State

| File | Magic Number Count |
|------|-------------------|
| `src/vm/interpreter/builtins.rs` | 206 |
| `src/vm/interpreter/native_loader.rs` | 89 |
| `src/vm/interpreter/property_access.rs` | 62 |
| `src/runtime_env/native_fns/fetch_fns.rs` | 40 |
| `src/runtime_env/native_fns/iterator_fns.rs` | 30 |
| `src/runtime_env/native_fns/url_fns.rs` | 13 |
| `src/vm/interpreter/iterators.rs` | 6 |
| `src/runtime_env/native_fns/websocket_fns.rs` | 4 |
| `src/runtime_env/native_fns/intl_fns.rs` | 3 |
| `src/runtime_env/native_fns/crypto_fns.rs` | 2 |
| **Total** | **455** |

## Proposed Solution

### Step 1: Create `src/runtime_env/native_fns/constants.rs`

Define every index as a named constant in one file. Group by category:

```rust
// Console (0-3)
pub const CONSOLE_LOG: usize = 0;
pub const CONSOLE_WARN: usize = 1;
pub const CONSOLE_ERROR: usize = 2;
pub const CONSOLE_INFO: usize = 3;

// Object (4-7)
pub const OBJECT_KEYS: usize = 4;
pub const OBJECT_VALUES: usize = 5;
pub const OBJECT_ENTRIES: usize = 6;
pub const OBJECT_ASSIGN: usize = 7;

// JSON (8-9)
pub const JSON_PARSE: usize = 8;
pub const JSON_STRINGIFY: usize = 9;

// Global (10-17)
pub const PARSE_INT: usize = 10;
pub const PARSE_FLOAT: usize = 11;
pub const IS_NAN: usize = 12;
pub const IS_FINITE: usize = 13;
pub const SET_TIMEOUT: usize = 14;
pub const SET_INTERVAL: usize = 15;
pub const CLEAR_TIMEOUT: usize = 16;
pub const CLEAR_INTERVAL: usize = 17;

// Math (18-30)
pub const MATH_ABS: usize = 18;
// ... etc for all 394 entries

// Number.prototype (383-390)
pub const NUMBER_TO_FIXED: usize = 383;
pub const NUMBER_TO_STRING: usize = 384;
pub const NUMBER_VALUE_OF: usize = 385;
pub const NUMBER_TO_EXPONENTIAL: usize = 386;
pub const NUMBER_TO_PRECISION: usize = 387;
pub const NUMBER_IS_INTEGER: usize = 388;
pub const NUMBER_IS_SAFE_INTEGER: usize = 389;
pub const PARSE_FLOAT_NUM: usize = 390;

// Boolean.prototype (391-392)
pub const BOOLEAN_TO_STRING: usize = 391;
pub const BOOLEAN_VALUE_OF: usize = 392;

// String.matchAll (393)
pub const STRING_MATCH_ALL: usize = 393;
```

### Step 2: Add compile-time length assertion

In `constants.rs`:
```rust
/// Total number of entries in NATIVE_TABLE.
/// Update this constant when adding/removing entries.
pub const NATIVE_TABLE_LEN: usize = 394;
```

In `mod.rs` (NATIVE_TABLE definition):
```rust
use super::constants::NATIVE_TABLE_LEN;

const _: () = assert!(
    NATIVE_TABLE.len() == NATIVE_TABLE_LEN,
    "NATIVE_TABLE length mismatch — update NATIVE_TABLE_LEN in constants.rs"
);
```

### Step 3: Replace magic numbers across all files

For each file, replace `Value::NativeFunction(286)` → `Value::NativeFunction(constants::FS_READ_FILE_SYNC)`, etc.

**Files to modify (in order of impact):**

1. **`src/vm/interpreter/builtins.rs`** (206 changes) — Global objects: console, Object, JSON, Math, Promise, Error, Date, RegExp, Map, Set, WeakMap, WeakSet, URL, Headers, Request, Response, fetch, crypto, assert, child_process, Number, Reflect, Proxy, Symbol, generator functions

2. **`src/vm/interpreter/native_loader.rs`** (89 changes) — Module registration for fs, path, process, os, buffer, intl, events, crypto, assert, child_process, url

3. **`src/vm/interpreter/property_access.rs`** (62 changes) — Prototype method dispatch: Function.prototype, Promise, Symbol, Date, RegExp, Map, Set, WeakMap, WeakSet, TypedArray, generator, Number, Boolean, String.matchAll

4. **`src/runtime_env/native_fns/fetch_fns.rs`** (40 changes) — Internal callbacks referencing other functions by index

5. **`src/runtime_env/native_fns/iterator_fns.rs`** (30 changes) — Iterator helper callbacks

6. **`src/runtime_env/native_fns/url_fns.rs`** (13 changes) — URL static methods

7. **`src/vm/interpreter/iterators.rs`** (6 changes) — Array iterator dispatch

8. **`src/runtime_env/native_fns/websocket_fns.rs`** (4 changes)

9. **`src/runtime_env/native_fns/intl_fns.rs`** (3 changes)

10. **`src/runtime_env/native_fns/crypto_fns.rs`** (2 changes)

### Step 4: Update mod.rs

```rust
mod constants;

pub use constants::*;
```

Remove all inline comments like `// Number.prototype methods (383-389)` since constants self-document.

### Step 5: Add a smoke test

In `tests/` or as an inline test in `constants.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_table_length_matches_constant() {
        assert_eq!(
            super::super::NATIVE_TABLE.len(),
            NATIVE_TABLE_LEN,
            "NATIVE_TABLE has {} entries but NATIVE_TABLE_LEN = {}",
            super::super::NATIVE_TABLE.len(),
            NATIVE_TABLE_LEN
        );
    }
}
```

## Migration Strategy

1. Create `constants.rs` with all 394 constants (one-time effort)
2. Run `cargo build` — should compile with no changes yet
3. Replace magic numbers file by file, starting with the highest-impact files
4. After each file, run `cargo build` to verify no breakage
5. Add the compile-time assertion last
6. Run `cargo test` and `cargo clippy`

## Future Prevention

- **Adding a new function**: Add the constant to `constants.rs`, increment `NATIVE_TABLE_LEN`, add the entry to `NATIVE_TABLE`, then use the constant everywhere. The compile-time assertion catches mismatches.
- **Reordering functions**: Update the constant values. The assertion catches missing updates.
- **Code review**: PRs touching NATIVE_TABLE should verify all constants match their positions.
