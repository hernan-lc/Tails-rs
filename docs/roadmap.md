# Roadmap

> Based on current implementation status. Contributions welcome!

## v0.2.0 — Correctness & Quality

### Must Fix
- [ ] **Reflect API** — All 13 methods in `src/objects/js_proxy.rs` are stubs returning dummy values. Implement `get`, `set`, `has`, `deleteProperty`, `ownKeys`, `getOwnPropertyDescriptor`, `defineProperty`, `getPrototypeOf`, `setPrototypeOf`, `isExtensible`, `preventExtensions`, `apply`, `construct`
- [ ] **CI Test Pipeline** — No `cargo test` runs on push/PR. Add GitHub Actions workflow for test + clippy
- [ ] **Generators benchmark** — Returns 0.00ms (likely failing silently). Investigate and fix

### Missing Tests
- [ ] WebSocket module — No Rust test harness
- [ ] OS module — No dedicated `os_module.rs`
- [ ] child_process — No tests for `execSync`, `exec`, `spawn`
- [ ] crypto — No tests for `randomBytes`, `randomUUID`, `createHash`
- [ ] events — No tests for `EventEmitter`
- [ ] CLI — No tests for `tails build`, `tails clean`, `--watch`, `--env-file`

### Missing Examples
- [ ] WebSocket client example
- [ ] Async/await patterns example
- [ ] CommonJS require() example
- [ ] child_process usage example

## v0.3.0 — Native Module Polish

### DTS Generation
- [ ] Auto-generate `.d.ts` type definitions for native modules (currently only `tails-validator` has them)
- [ ] `tails build` should output `dist/<name>.d.ts` alongside `.so`

### Module Fixes
- [ ] **process module** (`modules/process`) — Has bare Rust functions but no `#[tails_module]` macro annotation. The runtime has built-in implementations in `src/runtime_env/` but the standalone crate is incomplete
- [ ] **websocket module** (`modules/websocket`) — Rust-only struct with no FFI bridge. Needs `#[tails_module]` annotation for `.native` import support

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
