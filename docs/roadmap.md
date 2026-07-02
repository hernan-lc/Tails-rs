# Roadmap

> Based on current implementation status. Contributions welcome!

## v0.2.0 — Correctness & Quality

### Must Fix
- [x] **Reflect API** — Removed dead stub code from `js_proxy.rs`. All 13 methods were already fully implemented in `reflect_fns.rs` (get, set, has, deleteProperty, ownKeys, getOwnPropertyDescriptor, defineProperty, getPrototypeOf, setPrototypeOf, isExtensible, preventExtensions, apply, construct)
- [x] **EventEmitter prototype inheritance** — Fixed constructor in `events_fns.rs` to use `this` (which has the correct prototype from the VM) instead of creating a new object with `prototype: None`. Added EventEmitter (index 312) to `find_native_prototype` in `calls.rs` to look up the prototype from the module registry
- [x] **CI Test Pipeline** — `.github/workflows/ci.yml` runs on push to `master`, PRs, and manual dispatch. Five jobs: `build`, `test` (default features, `--test-threads=1`), `test-no-default-features`, `lint` (clippy, advisory), `fmt` (advisory). Cargo cache via `Swatinem/rust-cache@v2` with one shared anchor. Status badge in `README.md`.
- [ ] **Generators benchmark** — Returns 0.00ms (likely failing silently). Investigate and fix

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

## v0.4.0 — Performance

Critical hotspots from benchmarks (vs Node.js):

| Area | Current | Target | Notes |
|------|---------|--------|-------|
| string_concat | 89x slower | <5x | String interning or rope data structure |
| closures | 415x slower | <10x | Environment capture optimization |
| map_set | 74x slower | <5x | Use hashbrown/FxHashMap internally |
| loops | 44x slower | <5x | Bytecode dispatch optimization |
| promises | 31x slower | <5x | Microtask queue optimization |
| regexp | 17.5x slower | <5x | Regex compilation caching |

### Strategy
- [ ] Profile closures — likely environment clone overhead on capture
- [ ] Profile string_concat — likely GC/allocation pressure
- [ ] Replace internal HashMap with FxHashMap for Map/Set
- [ ] Optimize bytecode dispatch (inline caching, threaded dispatch)
- [ ] Promise microtask queue batching
- [ ] Consider single-pass JIT tier for hot loops

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
