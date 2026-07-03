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

#### Built-in (compiled into the binary)
- [x] `console` — log, warn, error, info, table, dir, group, groupEnd, groupCollapsed, time, timeEnd, assert, clear
- [x] `events` — EventEmitter with on, emit, off, listenerCount
- [x] `assert` — strictEqual, ok, equal, deepEqual
- [x] `buffer` — alloc, from, concat, isBuffer, isEncoding, byteLength, transcode, toString, write, slice, copy, fill, compare, equals, indexOf
- [x] `crypto` — randomBytes, randomUUID, createHash, hashUpdate, hashDigest
- [x] `child_process` — execSync, exec, spawn
- [x] `intl` — DateTimeFormat, NumberFormat

#### Feature-gated (enabled by default)
- [x] `fs` — readFileSync, writeFileSync, existsSync, mkdirSync, readdirSync, statSync, unlinkSync, rmSync, copyFileSync, renameSync, appendFileSync, createReadStream, watch
- [x] `fs/promises` — readFile, writeFile, readdir, stat, mkdir, unlink, copyFile, rename, appendFile, exists (promise-style)
- [x] `path` — join, resolve, basename, dirname, extname, relative, isAbsolute, normalize, parse, format, sep, delimiter
- [x] `process` — exit, cwd, chdir, platform, arch, pid, env, argv, hrtime, nextTick, kill, uptime, memoryUsage, on
- [x] `os` — platform, arch, cpus, totalmem, freemem, uptime, hostname, type, release, homedir, tmpdir, endianness, loadavg
- [x] `http` — createServer
- [x] `net` — createConnection (TCP client)
- [x] `url` — URL constructor, URLSearchParams, canParse, parse, toJSON, fileURLToPath

#### cdylib modules (loadable via `import x from "./x.native"`)
- [x] `tails-fs` — filesystem operations (sync + async + streaming)
- [x] `tails-fs-promises` — promise-style filesystem API
- [x] `tails-path` — path utilities
- [x] `tails-process` — process control
- [x] `tails-os` — OS info
- [x] `tails-websocket` — WebSocket client (async via tokio)
- [x] `tails-validator` — Zod-like validation library

#### Global objects (no import needed)
- [x] `Object`, `Array`, `Map`, `Set`, `WeakMap`, `WeakSet`
- [x] `Promise`, `Proxy`, `Reflect`, `Symbol`
- [x] `Date`, `RegExp`, `Math`, `Number`, `BigInt`, `String`, `Boolean`
- [x] `Error`, `TypeError`, `ReferenceError`, `SyntaxError`, `RangeError`
- [x] `JSON`, `URL`, `URLSearchParams`, `Headers`, `Request`, `Response`, `fetch`
- [x] `TypedArray` family (Int8Array through BigUint64Array)
- [x] `Generator`, `WebSocket`, `Buffer`, `process`

#### Implemented
- [x] `util` — `format`, `inspect`, `promisify`, `callbackify`
- [x] `events` — Expand EventEmitter (prependListener, once, MaxListeners, removeAllListeners, eventNames, setMaxListeners, getMaxListeners, prependOnceListener)
- [x] `timers` — `setImmediate`, `clearImmediate`
- [x] `querystring` — `parse`, `stringify`, `encode`, `decode`
- [x] `stream` — Readable/Writable/Transform/PassThrough streams
- [x] `zlib` — Compression/decompression (gzip, deflate, inflate + sync variants)
- [x] `tls` — TLS/SSL support (connect, createSecureContext, createServer)
- [x] `dns` — DNS resolution (resolve, lookup, resolve4, resolve6, resolveMx)

### API Completeness
- [x] `Buffer` — Added `isEncoding(enc)`, `transcode(src, fromEnc, toEnc)`, and the `byteLength(string, encoding)` encoding overload. `transcode` supports `utf8` ⇄ `latin1` / `ascii` / `hex` / `base64` / `base64url`; `utf16le` is recognised as a valid encoding name but the actual transcoding returns `null` (queued for a follow-up). Unknown encodings also return `null`. See `src/runtime_env/native_fns/buffer_fns.rs` (`is_supported_encoding`, `native_buffer_is_encoding`, `native_buffer_transcode`).
- [x] `process` — Added `kill(pid, signal)`, `on('exit', handler)`, `memoryUsage()`, `uptime()`. `kill` accepts both POSIX signal names (`"SIGTERM"`, `"SIGKILL"`, …) and raw integers (e.g. `9`); signal 0 is the standard "existence check". Exit handlers are stored in a process-global `Mutex<Vec<Value>>` and invoked in LIFO order (matching Node) before the process actually terminates. `memoryUsage` returns `{rss, heapTotal, heapUsed, external, arrayBuffers}` with `rss` read from `/proc/self/status` on Linux and `ps -o rss=` on macOS. `uptime` returns wall-clock seconds since first call. The `process` module gains a `libc = "0.2"` dependency to call `kill(2)`.
- [x] `fs` — `promises` API + `createReadStream` + `watch` are now shipped as `modules/fs-promises/` (cdylib) and additions to `modules/fs/src/lib.rs`. The `fs/promises` cdylib exposes `readFile`, `writeFile`, `readdir`, `stat`, `mkdir`, `unlink`, `copyFile`, `rename`, `appendFile`, `exists` with a uniform `{ok, value|error}` JSON envelope that works with `await` (same shape as the runtime's built-in `native_fs_read_file` etc.). The `fs` cdylib gains `createRead_stream` / `stream_read` / `stream_close` (chunked reads with base64-encoded payloads) and `watch` / `watch_poll` / `watch_close` (polling-based directory snapshot diff producing `create` / `modify` / `delete` events). 9 new integration tests in `tests/fs_promises_module.rs` + 5 in `tests/fs_module.rs` cover both surfaces. `dist/tails-fs.d.ts` and `dist/tails-fs-promises.d.ts` are auto-generated (with module-scoped DTS symbol filtering so cross-cdylib static links don't bleed into the wrong package).
- [x] `path` — `parse()` and `format()` are already shipped in `modules/path/src/lib.rs`; the roadmap entry was stale. `tests/api_completeness_v050.rs` now has regression tests covering the full `parse()` round-trip and `format()` → `parse()` round-trip for `/home/user/file.txt`.

### Implemented (v0.5.0 Pass 1)

First round of `Buffer` / `process` / path work toward Node.js
compatibility. 15 new integration tests in
`tests/api_completeness_v050.rs` cover every new public function
(skipping gracefully when the corresponding `tails-*` cdylib is not in
`dist/`). Six new entries in `src/runtime_env/native_fns/constants.rs`
(406–411) and `NATIVE_TABLE_LEN` bumped 406 → 412.

- [x] **Buffer globals** — `Buffer` is now a Node-style bare global (no import needed) in `src/vm/interpreter/builtins.rs:921-941`. The same property map that the native-module factory builds is hoisted into `Interpreter.globals`; `modules.rs` adds `Buffer` to the set of globals that survive `eval_module`'s `saved_globals` restoration.
- [x] **process global** — Same treatment as `Buffer`: when the `process` feature is enabled, `process` is exposed as a bare global in addition to the `import process from "./process.native"` route. `src/vm/interpreter/builtins.rs:946-961` and `src/vm/interpreter/modules.rs:135-136`.
- [x] **`props!` macro** — New `macro_rules! props` in `src/vm/interpreter/heap_types.rs:10-22` builds an `FxHashMap<String, Value>` from `key => value` pairs. Used to collapse the 12 manual `props.insert(...)` calls in `url_fns.rs` into a single block. Available as `crate::props!`.
- [x] **`process.kill` FFI** — `modules/process/src/lib.rs` gains a `kill(pid, signal)` Rust helper (calls `libc::kill(2)`) plus a `parse_signal` table covering all POSIX signal names. The cdylib FFI export accepts a `tails_abi::NativeValue` for the signal so that both string and numeric JS arguments reach the libc call with the right conversion (the macro's `FromNativeValue` impl would otherwise collapse numbers to `""` and dispatch a default `SIGTERM`).
- [x] **`process.uptime` / `process.memoryUsage` FFI** — New `process_uptime_secs()` and `memory_usage()` Rust helpers plus their FFI exports (`uptime` returns a `f64`; `memory_usage` returns a `serde_json` object string parsed by the FFI wrapper).
- [x] **Buffer `transcode` round-trips** — Verified by `test_buffer_transcode_utf8_to_hex_to_utf8_roundtrip`, `test_buffer_transcode_base64_roundtrip`, and `test_buffer_transcode_unknown_encoding_returns_null` in `tests/api_completeness_v050.rs`.
- [x] **Buffer `byteLength` encoding overload** — `Buffer.byteLength("Hi", "hex")` and 6 other encodings all return `2`; `Buffer.byteLength("ñ")` correctly returns `2` (UTF-8 multi-byte). Covered by `test_buffer_byte_length_*`.

### Strategy (remaining)
- [ ] `Buffer.transcode` — actually implement `utf16le` ⇄ `utf8` (currently returns `null`)
- [ ] `process.kill` — Windows support (`GenerateConsoleCtrlEvent` / `OpenProcess` + `TerminateProcess`)
- [ ] Refactor remaining `props.insert(...)` sites in the codebase to use `crate::props!` (currently only `url_fns.rs` was migrated)
- [ ] `fs.promises` / `fs.createReadStream` / `fs.watch` — done as of the `fs` entry above

## v1.0.0 — Stability

- [ ] Audit all unsafe code — verify safety invariants
- [ ] Fuzzing harness for lexer + parser
- [ ] Documentation site (mdbook or similar)
- [ ] npm package for `tails` CLI
- [ ] Windows + macOS CI testing (currently Linux-only)
- [ ] Memory leak audit with valgrind/ASAN
