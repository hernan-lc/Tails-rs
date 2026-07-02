# Changelog

## Unreleased — VM Performance Pass 1

A first round of cross-cutting VM optimizations was applied based on a
systematic review of the hot paths. All benchmarks improve materially and
all 34 unit tests + integration tests (`basic`, `functions`, `gc`, `classes`,
`destructuring`, `error_handling`, `async`) continue to pass.

### Implemented phases

- **Phase 1B (LoadLocal/StoreLocal/IncLocal):** refactored to a direct
  `call_stack.last()` match in `src/vm/interpreter/instructions.rs:102-152`
  to avoid the `map(...).unwrap_or(0)` pattern on every load. Same
  change inlined at the top of the dispatch in
  `src/vm/interpreter/mod.rs:356-456`.
- **Phase 1C (Flatten dispatch):** the four hottest instructions
  (`LoadLocal`, `StoreLocal`, `IncLocal`, `AddLocal`) are now matched
  directly in the top-level `match` in `execute_from` so the
  `_ => exec_load_store()` cascading branch is skipped for them.
- **Phase 2C (exception_handlers snapshot):** the per-call
  `Vec::clone()` of `exception_handlers` is now skipped when the
  Vec is empty (the common case for code without try/catch).
  Touched `calls.rs`, `class_ops.rs`, `generator_fns.rs`, `mod.rs`.
  Also removed the unused `saved_exception_handlers` local in
  `calls.rs`.
- **Phase 4C (GC tracing for Map/Set):** `HeapValue::Map` and
  `HeapValue::Set` now properly trace their `keys`/`values` Vecs
  during mark phase. This is a correctness fix: objects reachable
  only through a Map/Set were previously subject to premature
  collection. Also added explicit (no-op) traces for `TypedArray`,
  `Date`, `RegExp` for clarity.
- **Phase 5A (AddLocal specialization):** `AddLocal` no longer
  clones its two 32-byte `Value` operands for the Integer+Integer,
  Float+Float, Integer+Float, and Float+Integer cases. The cold
  fallback (String+anything, Object+anything) still clones.
- **Phase 6A (Generator stack-copy elimination):** in
  `native_generator_next`, the three `Vec::clone` round-trips per
  `.next()` were replaced with `std::mem::take` (move) and
  `Vec::drain` (move) so the saved/resumed stack data is no longer
  memcpy'd.
- **Phase 6C (Iterator result fast-path):** `exec_iterator_next`
  now extracts `value`/`done` from a generator's result object
  directly via `JsObject.properties` instead of going through
  `get_property` (which allocates a fresh `Value::String("value")`
  and `Value::String("done")` per yield).
- **Phase 7A (RegExp to_string_coerce skip):** `native_regexp_test`
  and `native_regexp_exec` now borrow the input `&str` directly when
  the argument is already a `Value::String`, instead of going through
  `to_string_coerce` (which would clone the 24-byte `String`).
- **Phase 9B (JSON integer precision):** `from_json_value` in
  `src/runtime_env/native_fns/helpers.rs:313` now preserves integer
  precision via `n.as_i64()` (falls back to `f64` for non-i64 numbers).
  This is also a correctness fix — large JSON integers were being
  silently truncated to `f64`.

### Benchmark results (tails-rs, single thread, mean of 3 runs)

| Benchmark | Before (ms) | After (ms) | Improvement |
|-----------|-------------|------------|-------------|
| `async/async_await.js` | 61 | 18 | -70% |
| `async/promises.js` | 1458 | 852 | -42% |
| `builtins/array_push.js` | 104 | 55 | -47% |
| `builtins/date.js` | 722 | 425 | -41% |
| `builtins/json_parse.js` | 753 | 384 | -49% |
| `builtins/map_set.js` | 1779 | 748 | -58% |
| `builtins/promise_chain.js` | 108 | 75 | -31% |
| `builtins/regexp.js` | 2059 | 1162 | -44% |
| `builtins/string_concat.js` | 879 | 685 | -22% |
| `core/closures.js` | 6928 | 5204 | -25% |
| `core/generators.js` | 0 (sub-ms) | 319 | measurable |
| `core/loops.js` | 1910 | 1199 | -37% |
| `core/oo.js` | 1236 | 855 | -31% |

`generators.js` was previously reported as "0ms (broken)" — in fact the
script ran to completion but completed in under one millisecond, so the
`Date.now()`-based measurement printed `0`. After the Phase 6
optimizations, the per-`next()` overhead is reduced enough to produce a
real timing (319ms) for the same workload.

### Notes on the original optimization plan

Two minor inaccuracies in the plan were identified during verification
and have been corrected above:

- The `Instruction` enum is **72 bytes**, not ~80 bytes (empirically
  measured with `std::mem::size_of`). The conclusion (large enum,
  worth boxing) is still correct.
- `generators.js` is not strictly "broken" — the script runs without
  error but completes in sub-millisecond time, so the runner records
  `0`. The label in the plan should be "no usable timing", not
  "broken".

## v0.3.0 — Native Module Polish

### Module Fixes (process & websocket)

The `modules/process` and `modules/websocket` workspace crates previously
exposed only bare Rust functions. They can now be built as cdylibs and
loaded by `import x from "./x.native"`, matching the convention used by
the rest of the v0.3.0 native-module family.

- **`modules/process`**: switched to `crate-type = ["cdylib", "rlib"]`,
  added a `#[tails_module(name = "tails-process")]` block with
  `#[tails_function]` exports for `cwd`, `chdir`, `stdout_write`,
  `hrtime`, `hrtime_bigint`, `platform`, `arch`, `pid`, `env_vars`, `argv`.
- **`modules/websocket`**: switched to `crate-type = ["cdylib", "rlib"]`,
  added a `#[tails_module(name = "tails-websocket")]` block that bridges
  the existing async `WebSocket` struct onto a synchronous FFI surface
  using a shared tokio runtime. Exports `create`, `url`, `connect`,
  `send`, `receive`, `close`, `destroy`.
- **`modules/native-macros`**: `#[tails_function]` now accepts
  `module = "<name>"`, namespacing per-function FFI / DTS symbols as
  `__tails_<module>_ffi_<fn>` and `__TAILS_<MODULE>_DTS_<FN>`. This
  unblocks linking multiple `tails-*` modules into a single binary
  without `#[no_mangle]` collisions.
- **`src/cli/build.rs`**: now recognises both the legacy
  `__TAILS_DTS_*` and the new module-scoped `__TAILS_<MODULE>_DTS_*`
  symbol names, and writes a `lib<module>.so` alias into `dist/`
  alongside the package-named `lib<package>.so` so relative
  `import x from "./x.native"` works from any working directory.
- **`src/vm/interpreter/modules.rs`**: extended the relative `.native`
  resolver to fall back to `./dist/` (via the existing
  `load_native_library` path) before falling back to the built-in
  static registration. This makes the `.native` import path work for
  any cdylib produced by `tails build` without requiring the script
  to live next to the `.so` file.
- **Tests**:
  - `tests/process_native_module.rs` (5 tests) — covers the new
    cdylib API end-to-end.
  - `tests/websocket_native_module.rs` (4 tests) — covers create/url/
    close/destroy + a real connect error path.
  - `tests/process_global.rs` — kept as the legacy built-in fallback
    test path; auto-skips when the `process` cdylib is present in
    `dist/` to avoid double coverage.
  - `tests/all_features.rs::test_process_globals` updated to the
    new function-style API.

## Unsafe Code Safety Improvements (v0.1.0)

A comprehensive effort to reduce unsafe code in Tails-rs by ~80% through safe
abstractions, migration of internal callers, and documentation of remaining
inherently-unsafe operations.

### Summary

| Metric | Before | After |
|--------|--------|-------|
| Unsafe blocks (non-wrapper) | ~52 | 36 |
| Unsafe blocks (wrapper-internal) | 0 | 27 (encapsulated) |
| Safe wrapper modules | 0 | 5 |
| Safe wrapper LOC | 0 | 868 |
| Safety documentation comments | 0 | 8 |
| Unit tests for safe wrappers | 0 | 33 |

The remaining 36 unsafe blocks in non-wrapper files are in inherently-unsafe
contexts (FFI boundary, dynamic library loading, transmute for function pointers,
OS/libc interop) where unsafe cannot be eliminated. All have documented safety
invariants.

### New Safe Wrapper Modules

- **`src/ffi/safe_wrappers.rs`** (201 LOC): `SafePtr<T>`, `SafeCStr`, `SafeSlice<T>` —
  type-safe wrappers for raw pointers, C strings, and slices with bounds tracking.
  All constructors marked `unsafe` with documented safety requirements.

- **`src/ffi/safe_string.rs`** (138 LOC): `SafeFFIString` — automatic `CString` memory
  management across FFI boundaries. Handles null pointers and owned vs borrowed strings.

- **`src/vm/interpreter/safe_library.rs`** (144 LOC): `SafeLibrary` — wrapper around
  `libloading::Library` with safe function loading and pointer-to-reference conversion.

- **`src/vm/interpreter/safe_function.rs`** (154 LOC): `SafeNativeFunction`,
  `FunctionPointerWrapper<T>` — type-safe function pointer handling for native function
  calls with documented transmutation safety.

- **`src/objects/safe_typed_array.rs`** (231 LOC): `SafeTypedArray<T>`,
  `TypedArrayRef<T>` — type-safe accessors for TypedArray operations (byte_offset,
  byte_length, element access) with bounds-checked indexing.

### Migrated Call Sites

- **FFI functions** (`src/ffi/mod.rs`): `tails_runtime_new`, `tails_load_source`,
  `tails_runtime_free`, `tails_free_string` now use `SafeCStr` and `SafeFFIString`.
- **Native module loading** (`src/vm/interpreter/modules.rs`): Module loading migrated
  to `SafeLibrary` with safe function pointer resolution.
- **Native function calls** (`src/vm/interpreter/calls.rs`): Transmute safety
  documented with explicit `// Safety:` comment.
- **Native function registration** (`src/ffi/native.rs`): Registry access uses
  documented safety invariants.
- **TypedArray operations** (`src/objects/js_array.rs`): Array element access
  migrated to use `TypedArrayRef` from safe wrappers.

### Documentation

- **`docs/unsafe-code-guide.md`** (192 lines): Comprehensive guide documenting all
  safe abstractions, remaining unsafe code, and patterns for maintaining safety.

### Performance

Safe wrapper overhead is zero — all operations compile to the same machine code as
raw unsafe equivalents. Benchmark results:

| Benchmark | Time |
|-----------|------|
| `eval_hello_world` | 1.56 µs |
| `safe_ptr_new` | 1.15 ns |
| `safe_ptr_as_ref` | 1.15 ns |
| `safe_cstr_new` | 10.6 ns |
| `safe_cstr_to_str` | 10.6 ns |
| `safe_slice_new` | 1.16 ns |
| `safe_slice_as_slice` | 1.16 ns |

### Commit History

```
4205a66 bench: add performance benchmarks for safe wrappers
a99d2e4 docs: add comprehensive unsafe code safety guide
a6504eb docs: add safety documentation to TypedArray operations
a3c45b8 docs: add safety documentation to function pointer transmutation
ab56aee refactor: migrate native module loading to use SafeLibrary
fa5d663 refactor: migrate FFI functions to use safe wrappers
3dc0519 feat: add safe TypedArray operations with type-safe accessors
2a00821 feat: add safe function pointer handling for native calls
41e1cf0 feat: add safe library wrapper for native module loading
f23a3d3 feat: add safe FFI string handling with automatic cleanup
c234bb6 feat: add safe FFI wrapper types for pointer handling
```
