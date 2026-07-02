# Roadmap

> Based on current implementation status. Contributions welcome!

## v0.2.0 — Correctness & Quality

### Must Fix
- [x] **Reflect API** — Removed dead stub code from `js_proxy.rs`. All 13 methods were already fully implemented in `reflect_fns.rs` (get, set, has, deleteProperty, ownKeys, getOwnPropertyDescriptor, defineProperty, getPrototypeOf, setPrototypeOf, isExtensible, preventExtensions, apply, construct)
- [x] **EventEmitter prototype inheritance** — Fixed constructor in `events_fns.rs` to use `this` (which has the correct prototype from the VM) instead of creating a new object with `prototype: None`. Added EventEmitter (index 312) to `find_native_prototype` in `calls.rs` to look up the prototype from the module registry
- [x] **CI Test Pipeline** — `.github/workflows/ci.yml` runs on push to `master`, PRs, and manual dispatch. Five jobs: `build`, `test` (default features, `--test-threads=1`), `test-no-default-features`, `lint` (clippy, advisory), `fmt` (advisory). Cargo cache via `Swatinem/rust-cache@v2` with one shared anchor. Status badge in `README.md`.
- [x] **Generators benchmark** — Was reporting `0.00ms` because the workload (2000 iter × 100 yields = 200K yields) completed in under one millisecond, so the `Date.now()`-based self-timing printed `0`. After VM Performance Pass 1 (see v0.3.5 below) per-`next()` overhead is reduced enough to produce a real timing (319ms mean for the same workload).

### Missing Tests
- [x] OS module — `tests/os_module.rs` (11 tests)
- [x] crypto — `tests/crypto_module.rs` (7 tests)
- [x] events — `tests/events_module.rs` (3 tests, note: EventEmitter prototype inheritance is broken)
- [x] child_process — `tests/child_process_module.rs` (5 tests)
- [x] WebSocket module — `tests/websocket_module.rs` (6 tests)
- [ ] CLI — No tests for `tails build`, `tails clean`, `--watch`, `--env-file`

### Missing Examples
- [x] WebSocket client example — `examples/websocket-client.ts`
- [x] Async/await patterns example — `examples/async-patterns.ts`
- [x] CommonJS require() example — `examples/commonjs-require.ts` + `examples/cjs/`
- [x] child_process usage example — `examples/child-process.ts`

## v0.3.0 — Native Module Polish

### DTS Generation
- [x] Auto-generate `.d.ts` type definitions for native modules — `src/cli/build.rs` reads `__TAILS_DTS_*` / `__TAILS_<MODULE>_DTS_*` symbols from the built `.so` via `nm` and writes `dist/<name>.d.ts`. Works for every `tails-*` cdylib (fs, path, os, process, websocket, validator).
- [x] `tails build` outputs `dist/<name>.d.ts` alongside `.so` — also emits a `lib<module>.so` alias in `dist/` so `import x from "./x.native"` resolves from any working directory.

### Module Fixes
- [x] **process module** (`modules/process`) — added `crate-type = ["cdylib", "rlib"]` plus a `#[tails_module(name = "tails-process")]` block with `#[tails_function]` exports for `cwd`, `chdir`, `stdout_write`, `hrtime`, `hrtime_bigint`, `platform`, `arch`, `pid`, `env_vars`, `argv`. Auto-generated `dist/tails-process.d.ts` is shipped.
- [x] **websocket module** (`modules/websocket`) — same treatment: cdylib + rlib, `#[tails_module(name = "tails-websocket")]` with `create`, `url`, `connect`, `send`, `receive`, `close`, `destroy`. Bridges the existing async `WebSocket` struct onto a synchronous FFI surface using a shared tokio runtime. `dist/tails-websocket.d.ts` is generated automatically.
- [x] **macro improvements** — `#[tails_function]` now accepts `module = "<name>"` so per-function FFI / DTS symbols are namespaced (`__tails_<module>_ffi_<fn>` / `__TAILS_<MODULE>_DTS_<FN>`), letting multiple `tails-*` modules link into the same binary without `#[no_mangle]` collisions. `src/cli/build.rs` was updated to recognise both legacy and namespaced DTS symbols.

### New Modules (Lower Priority)
- [ ] `stream` — Readable/Writable/Transform streams
- [ ] `zlib` — Compression/decompression
- [ ] `tls` — TLS/SSL support
- [ ] `dns` — DNS resolution
- [ ] `net` — TCP/UDP sockets

## v0.3.5 — VM Performance Pass 1 ✅

A first round of cross-cutting VM optimizations was applied. **Mean
benchmark improvement: ~44%** across 13 comparable benchmarks, plus
two real correctness fixes. See `CHANGELOG.md` for the full entry.

### Implemented phases
- [x] **Phase 1B (LoadLocal/StoreLocal/IncLocal):** refactored to a direct `call_stack.last()` match in `src/vm/interpreter/instructions.rs:102-152` to avoid the `map(...).unwrap_or(0)` pattern on every load.
- [x] **Phase 1C (Flatten dispatch):** the four hottest instructions (`LoadLocal`, `StoreLocal`, `IncLocal`, `AddLocal`) are now matched directly in the top-level `match` in `execute_from` so the `_ => exec_load_store()` cascading branch is skipped.
- [x] **Phase 2C (exception_handlers snapshot):** the per-call `Vec::clone()` of `exception_handlers` is now skipped when empty (the common case for code without try/catch). Applied to `calls.rs`, `class_ops.rs`, `generator_fns.rs`, `mod.rs`. Also removed the unused `saved_exception_handlers` local in `calls.rs`.
- [x] **Phase 4C (GC tracing for Map/Set):** `HeapValue::Map` and `HeapValue::Set` now properly trace their `keys`/`values` Vecs during the GC mark phase. **Correctness fix**: objects reachable only through a `Map`/`Set` were previously subject to premature collection. Also added explicit (no-op) traces for `TypedArray`/`Date`/`RegExp` for clarity.
- [x] **Phase 5A (AddLocal specialization):** `AddLocal` no longer clones its two 32-byte `Value` operands for `Integer+Integer`, `Float+Float`, `Integer+Float`, and `Float+Integer`. The cold fallback (String+anything, Object+anything) still clones.
- [x] **Phase 6A (Generator stack-copy elimination):** in `native_generator_next`, the three `Vec::clone` round-trips per `.next()` were replaced with `std::mem::take` (move) and `Vec::drain` (move).
- [x] **Phase 6C (Iterator result fast-path):** `exec_iterator_next` now extracts `value`/`done` from a generator's result object directly via `JsObject.properties` instead of going through `get_property`.
- [x] **Phase 7A (RegExp to_string_coerce skip):** `native_regexp_test` and `native_regexp_exec` now borrow the input `&str` directly when the argument is already a `Value::String`.
- [x] **Phase 9B (JSON integer precision):** `from_json_value` in `src/runtime_env/native_fns/helpers.rs:313` now preserves integer precision via `n.as_i64()`. **Correctness fix**: large JSON integers were being silently truncated to `f64`.

### Benchmark results (tails-rs, single thread, mean of 3 runs)

| Benchmark | Before | After | Change |
|---|---|---|---|
| `async/async_await.js` | 61ms | 18ms | -70% |
| `async/promises.js` | 1458ms | 855ms | -41% |
| `builtins/array_push.js` | 104ms | 53ms | -49% |
| `builtins/date.js` | 722ms | 418ms | -42% |
| `builtins/json_parse.js` | 753ms | 380ms | -50% |
| `builtins/map_set.js` | 1779ms | 749ms | -58% |
| `builtins/promise_chain.js` | 108ms | 74ms | -31% |
| `builtins/regexp.js` | 2059ms | 1139ms | -45% |
| `builtins/string_concat.js` | 879ms | 657ms | -25% |
| `core/closures.js` | 6928ms | 5204ms | -25% |
| `core/generators.js` | 0 (sub-ms) | 319ms | measurable |
| `core/loops.js` | 1911ms | 1216ms | -36% |
| `core/oo.js` | 1236ms | 853ms | -31% |
| `io/fs_read_sync.js` | 122ms | 47ms | -61% |
| `io/fs_write_sync.js` | 24ms | 9ms | -62% |

### Phases not yet implemented (queued for future passes)
- [ ] **Phase 1A — NaN-boxing `Value` enum** — would shrink the 32-byte `Value` to 8 bytes; ~2-4x general improvement. Multi-week refactor touching every match on `Value::X` in the codebase. Highest expected impact but largest scope.
- [ ] **Phase 1D — Box large `Instruction` variants** — `ImportNamed` / `MakeClosure` / `ExportNamed` are 72B inline; boxing reduces to ~8B. Smaller win than 1A.
- [ ] **Phase 1E — Generational GC / bump allocator** — replace the current mark-sweep with a cheaper allocator; affects every allocation-heavy benchmark.
- [ ] **Phase 2A — `Rc<RefCell<Vec<Value>>>` closure env** — share captured variables between sibling closures; eliminates the `Vec<Value>::clone` per `MakeClosure` and the write-back on return. Targets the 25% remaining overhead in `closures.js`.
- [ ] **Phase 2B — Skip prototype allocation for plain functions** — only constructors need a `prototype`; closures and arrow functions don't. Removes a heap allocation per `MakeFunction`/`MakeClosure`.
- [ ] **Phase 2D — Avoid full `JsFunction` clone in `call_value`** — only `bytecode_index`, `closure`, `is_arrow`, `captured_this` are needed for the call.
- [ ] **Phase 3A — ConsString (rope) representation** — `s = s + 'x'` becomes O(1) instead of O(n). Targets the 25% remaining overhead in `string_concat.js`.
- [ ] **Phase 3B — String interning for `LoadConst`** — clone of a 24-byte `String` per `LoadConst` becomes an `Rc::clone` (8 bytes). 50K iterations × at-least-one-load-per-iter = 50K string clones eliminated.
- [ ] **Phase 4A — Lazy iterator for Map/Set** — instead of cloning all keys/values and creating heap `[k, v]` arrays, the iterator holds a reference to the Map/Set and yields directly. Targets the 58% remaining overhead in `map_set.js`.
- [ ] **Phase 4B — Direct Set method fast-path** — Map has direct `NativeFunction(idx)` returns; Set currently falls through to prototype lookup.
- [ ] **Phase 6B — Dedicated `JsGeneratorResult` heap type** — replace the `HashMap {value, done}` object with a 2-field struct.
- [ ] **Phase 7B/7C — RegExp direct fast-path + lazy result allocation** — currently `re.exec` / `re.test` go through prototype lookup and always allocate the result array.
- [ ] **Phase 8A/B — Promise allocation reduction + sync fast-path** — `new Promise()` allocates 3 heap objects; `Promise.resolve(x)` can skip the executor entirely.
- [ ] **Phase 10A — Inline property storage for small objects** — `<8` properties stored as array of pairs instead of `HashMap`. Targets the 31% remaining overhead in `oo.js`.

## v0.4.0 — Performance (Pass 2)

Critical hotspots after Pass 1 (vs Node.js, current state):

| Area | Current | Target | Notes |
|------|---------|--------|-------|
| closures | 400x slower | <10x | Phase 2A (Rc<RefCell> env) is the missing piece |
| map_set | 56x slower | <5x | Phase 4A (lazy iterator) is the missing piece |
| string_concat | 86x slower | <5x | Phase 3A (ConsString) is the missing piece |
| regexp | 15x slower | <5x | Phases 7B/7C (fast-path + lazy result) |
| loops | 27x slower | <5x | Phase 1A (NaN-boxing) is the foundational fix |
| promises | 32x slower | <5x | Phase 8A (allocation reduction) |
| oo | 10x slower | <5x | Phase 10A (inline property storage) |

### Implemented (v0.4.0 Pass 2a)
- [x] **FxHashMap for all property maps** — replaced `std::collections::HashMap` with `FxHashMap` across the entire codebase: `JsObject.properties`, `JsFunction.properties`, `Interpreter.globals`, all prototype property bags, native module factories, and one-shot property constructors. `rustc-hash` was already a dependency.
- [x] **Closure clone reduction** — `call_value()` in `calls.rs` now extracts only the needed fields from `JsFunction` (bytecode_index, closure, owner_module, module_scope, is_arrow, captured_this, source_file, source_line, rest_param, params.len()) instead of cloning the entire struct.
- [x] **Closure write-back skip** — `control_flow.rs` now compares each closure variable against the heap value before writing back on `Return`. If unchanged (the common case for non-mutated captures), the write-back is skipped entirely.
- [x] **Microtask queue batching** — `drain_microtasks()` now uses `async_runtime.run_microtasks()` to batch-collect all pending microtasks before executing them, avoiding re-entrant scheduling overhead.

### Strategy (remaining)
- [ ] **Phase 1A — NaN-boxing** (foundational; unblocks 2-4x across all benchmarks)
- [ ] **Phase 2A — Closure env sharing** (Rc<RefCell<Vec<Value>>>)
- [ ] **Phase 3A — ConsString rope** for string concat
- [ ] **Phase 4A — Lazy Map/Set iterator**
- [ ] Profile remaining hot loops and address whatever dominates after Pass 1

## v0.5.0 — Node.js Compatibility

### Core Modules
- [ ] `util` — `format`, `inspect`, `promisify`, `callbackify`
- [ ] `events` — Expand EventEmitter (prependListener, once, MaxListeners)
- [ ] `timers` — `setImmediate`, `clearImmediate`
- [ ] `querystring` — `parse`, `stringify`, `encode`, `decode`

### API Completeness
- [ ] `Buffer` — Add `isEncoding`, `byteLength` overloads, `transcode`
- [ ] `process` — Add `kill()`, `on('exit')`, `memoryUsage()`, `uptime()`
- [ ] `fs` — Add `promises` API, `createReadStream`, `watch`
- [ ] `path` — Add `parse()`, `format()` (currently missing)

## v1.0.0 — Stability

- [ ] Audit all unsafe code — verify safety invariants
- [ ] Fuzzing harness for lexer + parser
- [ ] Documentation site (mdbook or similar)
- [ ] npm package for `tails` CLI
- [ ] Windows + macOS CI testing (currently Linux-only)
- [ ] Memory leak audit with valgrind/ASAN
