# Unsafe Code Audit & Safety Plan

**Date:** 2026-07-09  
**Status:** Implemented (Phases 0–4 landed in tree).  
**Scope:** All Rust sources under `src/`, `modules/`, and `benches/` (excluding `target/`, `node_modules/`).  
**Goal:** Either eliminate `unsafe` where possible, encapsulate remaining `unsafe` behind safe APIs, or document why it cannot be removed.

### Implementation summary (done)

| Item | Status |
|------|--------|
| TypedArray `NeBytes` (no ptr read/write) | Done |
| SharedValue → `Arc<Value>` rope children | Done |
| `nix` for os/process syscalls | Done |
| Typed `Vec<NativeFn>` (no call-site transmute) | Done |
| FFI / ABI Safe* migration | Done |
| CLI uses `SafeLibrary` | Done |
| Feature `jit` (default on) | Done |
| Feature `fast-json` (default off; safe serde path) | Done |
| `#![warn(unsafe_op_in_unsafe_fn)]` | Done |
| `docs/unsafe-code-guide.md` + allowlist script | Done |

---

## 1. Executive summary

| Category | Approx. sites | Can fully eliminate? | Recommended action |
|----------|---------------|----------------------|--------------------|
| **A. FFI C ABI boundary** | ~45 | **No** | Encapsulate + consistent wrappers |
| **B. Dynamic library loading** | ~15 | **No** | Use existing `SafeLibrary` everywhere |
| **C. JIT executable memory** | ~25 | **No** | Encapsulate in `CodeBuffer` only |
| **D. OS / libc syscalls** | ~7 | **Mostly yes** | Prefer safe crates (`nix`, `whoami`) |
| **E. TypedArray byte access** | ~4 | **Yes** | Use safe byte-order APIs |
| **F. ConsString / SharedValue** | ~4 | **Yes (tradeoff)** | `Rc<Value>` or keep + document |
| **G. simd-json** | 1 | **Yes (tradeoff)** | Safe API or serde-only |
| **H. Safe wrappers themselves** | ~40 | **N/A** | Keep: they *contain* unsafe |
| **I. Tests / benches** | ~20 | N/A | Keep for testing wrappers |

**Bottom line:** A runtime with FFI, native modules, and a JIT **cannot** be 100% free of `unsafe`. Prior work already cut non-wrapper `unsafe` ~80%. The remaining work is:

1. **Eliminate** the few pure-Rust sites that do not need `unsafe` (TypedArray, optional SharedValue/`Rc`, optional simd-json).
2. **Replace** raw libc with safe crates where APIs exist.
3. **Encapsulate** all remaining `unsafe` so call sites outside wrappers are zero or near-zero.
4. **Document** inherent `unsafe` with `# Safety` / `// Safety:` everywhere (some gaps remain).

**Target outcome:**  
- `#![deny(unsafe_op_in_unsafe_fn)]` where applicable.  
- Zero raw `unsafe` in “business logic” modules (interpreter opcodes, builtins that are not FFI).  
- All inherent `unsafe` only in: `ffi/`, `jit/code_buffer.rs`, `jit/trampoline.rs`, `abi/loader.rs`, wrapper modules.  
- CI: `cargo geiger` or custom count + allowlist.

---

## 2. Inventory by area

Counts are approximate “mentions” of the token `unsafe` (blocks, `fn`, `impl`, comments in tests).

| Area | Mentions | Primary files |
|------|----------|---------------|
| FFI + wrappers | ~60 | `src/ffi/{mod,native,conversions,safe_wrappers,safe_string}.rs` |
| JIT | ~26 | `src/vm/jit/{code_buffer,trampoline,mod}.rs` |
| Interpreter / native load | ~19 | `modules.rs`, `calls.rs`, `safe_library.rs`, `safe_function.rs` |
| Objects | ~12 | `strings.rs`, `js_array.rs`, `safe_typed_array.rs` |
| ABI / native macros | ~11 | `modules/abi/*`, `native-macros/*` |
| OS / process | ~7 | `modules/os`, `modules/process` |
| CLI / JSON | ~2 | `cli/build.rs`, `json_fns.rs` |
| Benches | ~8 | `benches/safe_wrappers.rs` |

Prior changelog claim: remaining non-wrapper blocks ~36 (inherently unsafe contexts). That still holds for **boundary** code; a few **non-boundary** sites (TypedArray, simd-json, SharedValue, raw libc) are still improvable.

> Note: `CHANGELOG.md` references `docs/unsafe-code-guide.md`, which is **not present** in the tree. This document supersedes it as the audit source of truth; a shorter maintainer guide can be split out later.

---

## 3. Classification: why each kind of `unsafe` exists

### Category A — FFI C ABI (`src/ffi/*`, `modules/abi`, native-macros)

**Why it exists**

- `extern "C"` APIs take `*mut T`, `*const c_char`, raw arg arrays.
- Ownership transfer: `Box::into_raw` / `Box::from_raw`, `CString::from_raw`.
- C strings have no length; UTF-8 validity is not guaranteed by the type system.

**Can we eliminate?**

| Operation | Eliminate? | Notes |
|-----------|------------|-------|
| `*mut Runtime` → `&mut Runtime` | No | Required at C boundary after null-check |
| `CStr::from_ptr` | No | Required for C strings |
| `from_raw_parts(args, argc)` | No | Required for C arg arrays |
| `Box::from_raw` / `into_raw` | No | Opaque handle ownership |
| `CString::from_raw` free | No | Caller-owned C string free |

**What we can do**

- Route **every** site through `SafePtr` / `SafeCStr` / `SafeSlice` / `SafeFFIString` (partially done; `tails_set_global` and several others still use raw `CStr::from_ptr`).
- Make public FFI entry points **safe** functions that only contain a thin `unsafe` block after null guards (pattern: check null → `unsafe { SafePtr::new(...).as_mut() }`).
- Keep `#allow(clippy::not_unsafe_ptr_arg_deref)` only if entry points stay `extern "C" fn` without `unsafe` keyword; prefer documenting why.

**Verdict:** **Not possible to remove.** Possible to reduce call-site noise and centralize invariants.

---

### Category B — Dynamic libraries (`libloading`, native modules)

**Why it exists**

- Loading `.so`/`.dylib`/`.dll` and resolving symbols is inherently unsafe (wrong signature → UB).
- `Library::new`, `get::<T>()`, holding symbols for the process lifetime.

**Sites**

- `src/vm/interpreter/safe_library.rs` — good encapsulation.
- `src/vm/interpreter/modules.rs` — still has `unsafe` for symbol lookup loop + `Box::from_raw(handle)`.
- `modules/abi/src/loader.rs` — parallel path, raw `libloading`.
- `src/cli/build.rs` — loads lib for `.d.ts` extraction.
- `calls.rs` — `transmute(func_ptr)` for dynamic native calls.

**Can we eliminate?**

| Operation | Eliminate? | Notes |
|-----------|------------|-------|
| `Library::new` / `get` | No | OS dynamic linker |
| `Box::from_raw(ModuleHandle*)` | No | Module init returns raw handle |
| `transmute` fn ptr → `extern "C" fn` | No* | *Can replace with typed storage |

**Improvements**

1. Store `NativeFn` (typed `extern "C" fn(...)`) in `dynamic_native_fns` instead of `usize` → **remove `transmute`** at call site.
2. Unify `abi::loader` and `SafeLibrary` so only one unsafe encapsulation path exists.
3. CLI `build.rs`: use `SafeLibrary` instead of open-coded `unsafe { Library::new }`.

**Verdict:** **Cannot remove.** Can remove **`transmute`** and duplicate loaders.

---

### Category C — JIT (`src/vm/jit/*`)

**Why it exists**

- Needs RWX or W^X memory: `mmap` / `VirtualAlloc` / `mprotect` / `VirtualProtect`.
- Emits machine code then calls it as a function pointer.
- `CodeBuffer` holds `*mut u8`; emit/patch use `write_unaligned` / `read_unaligned`.
- `call_jit` builds a `JitFrame` with raw pointers into interpreter stacks.

**Can we eliminate?**

| Operation | Eliminate? | Notes |
|-----------|------------|-------|
| Executable mapping | No | Required for JIT |
| Call emitted code | No | `transmute` entry or `extern "C"` cast |
| Emit/patch bytes | No* | *Could use `Vec<u8>` + platform crate, still unsafe underneath |
| Drop JIT entirely | Yes | Feature-gate JIT off → zero JIT unsafe |

**Improvements**

1. Optional crate: `region`, `memmap2`, or `cranelift-module`-style buffer — still `unsafe` inside.
2. Keep **all** platform syscalls only in `platform` module; public `CodeBuffer` methods stay safe (already mostly true for `emit*`).
3. Document `unsafe impl Send/Sync for CodeBuffer` with single-thread JIT assumption.
4. Feature `jit` default-off for builds that want minimal `unsafe` surface.

**Verdict:** **Not possible while JIT is enabled.** Possible to isolate and feature-gate.

---

### Category D — OS / libc (`modules/os`, `modules/process`)

**Why it exists**

- Node-compatible APIs: `os.hostname`, `os.getuid`, `process.kill`, etc.
- `std` does not expose all of these as pure safe Rust.

**Sites**

| Call | File | Safe alternative |
|------|------|------------------|
| `libc::gethostname` | `os` | `hostname` crate, or `nix::unistd::gethostname` |
| `libc::getppid/getuid/...` | `os` | `nix::unistd::{getuid, geteuid, getgid, getegid, getppid}` (safe wrappers) |
| `libc::kill` | `process` | `nix::sys::signal::kill` |

**Can we eliminate local `unsafe`?**

**Yes**, by depending on crates that encapsulate the syscall. The `unsafe` moves into dependencies (still present process-wide for `cargo geiger`, but **zero in our sources**).

**Tradeoff:** extra dependency; behavior must match Node.js semantics on each OS.

**Verdict:** **Possible to remove from Tails-rs sources** via `nix` (or similar). Not possible to remove from the process address space without dropping the APIs.

---

### Category E — TypedArray (`js_array.rs`, `safe_typed_array.rs`)

**Why it exists**

- Read/write unaligned integers/floats from a `Vec<u8>` buffer at a byte offset.

**Current pattern**

```rust
// after bounds check
unsafe {
    let ptr = self.buffer.as_ptr().add(byte_index) as *const T;
    Some(ptr.read_unaligned())
}
```

**Can we eliminate?**

**Yes.** Safe alternatives:

1. `T::from_ne_bytes` / `to_ne_bytes` on a fixed-size array copied from the slice after bounds check (100% safe, small copy).
2. `bytemuck` / `zerocopy` for POD types (may still use unsafe internally; call sites safe).
3. Keep bounds-checked helpers only inside one module with a single documented `unsafe` block.

**Verdict:** **Possible and recommended.** Low risk, clear safety win.

---

### Category F — ConsString / SharedValue (`src/objects/strings.rs`)

**Why it exists**

- Custom non-atomic refcounted pointer for rope children (`SharedValue`) to make clone O(1) without `Rc` overhead.
- `unsafe impl Send/Sync for ConsString` justified by “single VM thread only”.

**Can we eliminate?**

| Approach | Result |
|----------|--------|
| Use `Rc<Value>` instead of `SharedValue` | **Eliminates** `unsafe` deref/`from_raw`; slight cost, already single-threaded |
| Use `Arc` | Unnecessary if truly single-threaded |
| Keep custom RC | Document thoroughly; add debug asserts on refcount |

**Risk of current code:** `clone_ref` copies `count` into a new `Cell` incorrectly for multi-owner semantics? (worth a correctness review — each `SharedValue` holds its own `Cell` but shares `ptr`; Drop decrements **local** cell only → **possible refcount bug** if two clones drop independently). Recommend either switch to `Rc` or fix shared refcount layout (`Rc`-like single `Cell` next to the value).

**Verdict:** **Possible via `Rc`.** Strongly recommended for safety *and* likely correctness.

---

### Category G — simd-json (`json_fns.rs`)

**Why it exists**

- `simd_json::from_str` is `unsafe` because it requires a mutable exclusive buffer and may assume certain input properties for performance.

**Can we eliminate?**

| Approach | Result |
|----------|--------|
| Use only `serde_json::from_str` | **Zero unsafe**, slower parse |
| Use simd-json’s safer entry points if available for version | Check crate docs for 0.17 |
| Feature-gate `simd-json` | Default safe path; opt-in fast path |

**Verdict:** **Possible.** Tradeoff: performance on `JSON.parse`. Recommend feature `fast-json` with unsafe only behind it, default to serde.

---

### Category H — Safe wrappers (intentionally contain `unsafe`)

Modules:

- `src/ffi/safe_wrappers.rs` — `SafePtr`, `SafeCStr`, `SafeSlice`
- `src/ffi/safe_string.rs` — `SafeFFIString`, free on drop
- `src/vm/interpreter/safe_library.rs` / `safe_function.rs`
- `src/objects/safe_typed_array.rs`

**Verdict:** **Keep.** These are the **correct** place for `unsafe`. Goal is not zero wrappers, but zero *scattered* unsafe.

---

## 4. Site-by-site action table (non-wrapper production code)

| # | Location | Why unsafe | Avoid / change? | Priority |
|---|----------|------------|-----------------|----------|
| 1 | `ffi/mod.rs` many entry points | C ABI pointers | Encapsulate with Safe* fully | P1 |
| 2 | `ffi/native.rs` registry | C ABI | Same | P1 |
| 3 | `ffi/conversions.rs` string | C string in tag payload | SafeCStr | P1 |
| 4 | `abi/lib.rs` get_string / free / NativeString | C / raw slices | SafeCStr / SafeSlice | P1 |
| 5 | `abi/loader.rs` load + from_raw | dlopen | Reuse SafeLibrary | P1 |
| 6 | `interpreter/modules.rs` get_function + from_raw | symbols / ownership | Shrink unsafe block | P1 |
| 7 | `interpreter/calls.rs` transmute | untyped fn ptr | Store typed `NativeFn` | **P0** |
| 8 | `cli/build.rs` Library::new | dts extract | SafeLibrary | P2 |
| 9 | `native-macros` from_raw_parts | C args | Required; keep in macro | P3 |
| 10 | `jit/code_buffer.rs` | mmap/RX | Keep; optional feature | P2 |
| 11 | `jit/mod.rs` transmute entry | call code | Keep while JIT on | P2 |
| 12 | `jit/trampoline.rs` | call JIT | Keep | P2 |
| 13 | `os` gethostname + uids | libc | **`nix` / crates** | **P0** |
| 14 | `process` kill | libc | **`nix`** | **P0** |
| 15 | `js_array` TypedArray get/set | unaligned | **safe byte APIs** | **P0** |
| 16 | `strings` SharedValue | custom RC | **`Rc` or fix RC** | **P0** |
| 17 | `json_fns` simd_json | crate API | feature-gate / serde | P1 |
| 18 | `unsafe impl Send/Sync` (CodeBuffer, ConsString, NativeLibrary, Safe*) | cross-thread markers | Document + audit single-thread claims | P1 |

---

## 5. Phased plan

### Phase 0 — Policy & tooling (0.5–1 day)

1. Add `docs/UNSAFE_AUDIT_PLAN.md` (this file) as the living allowlist narrative.
2. Script or CI step: count `unsafe` outside an allowlist of directories:
   - `src/ffi/safe_*.rs`
   - `src/vm/jit/code_buffer.rs` (+ platform)
   - `src/vm/interpreter/safe_*.rs`
   - `src/objects/safe_typed_array.rs`
3. Enable `#![warn(unsafe_op_in_unsafe_fn)]` crate-wide; fix fallout.
4. Require `// Safety:` or `# Safety` on every new `unsafe` block (clippy `undocumented_unsafe_blocks` if available).

**Exit criteria:** CI fails if new non-allowlisted `unsafe` appears without review.

---

### Phase 1 — Eliminate avoidable `unsafe` (1–2 days) — **P0**

| Task | Change | Risk |
|------|--------|------|
| 1.1 TypedArray | Rewrite `get`/`set_value` with `from_ne_bytes` / `to_ne_bytes` + slice bounds | Low |
| 1.2 SharedValue | Replace with `Rc<Value>` **or** fix shared refcount layout + tests | Medium (perf + correctness) |
| 1.3 OS ids / hostname | Switch to `nix` (unix) safe wrappers; keep Windows stubs | Low |
| 1.4 process.kill | `nix::sys::signal::kill` | Low |
| 1.5 Dynamic native call | `Vec<NativeFn>` instead of `Vec<usize>` → delete `transmute` | Medium (ABI check) |

**Exit criteria:** Zero `unsafe` in `js_array` TypedArray paths, `modules/os`, `modules/process`, `calls.rs` transmute.

---

### Phase 2 — Encapsulate remaining boundary `unsafe` (2–3 days) — **P1**

| Task | Change |
|------|--------|
| 2.1 FFI completeness | Migrate all remaining `CStr::from_ptr` / raw deref in `ffi/mod.rs`, `native.rs`, `conversions.rs` to Safe* |
| 2.2 ABI crate | Same for `modules/abi` |
| 2.3 Single loader | `abi::loader` and interpreter both use `SafeLibrary` |
| 2.4 Module init | Small helper `unsafe fn take_module_handle(*mut ModuleHandle) -> Box<ModuleHandle>` with full Safety docs |
| 2.5 JSON | Feature `fast-json = ["simd-json"]`; default path serde only |
| 2.6 CLI | `build.rs` uses SafeLibrary |

**Exit criteria:** Non-wrapper production files contain only thin “call into wrapper” unsafe or none.

---

### Phase 3 — JIT isolation (1–2 days) — **P2**

| Task | Change |
|------|--------|
| 3.1 Feature `jit` | Gate `code_buffer`, `trampoline`, compile path |
| 3.2 Documentation | Module-level safety model for W^X and single-threaded compile |
| 3.3 Optional | Evaluate `memmap2` / `region` for clearer ownership |

**Exit criteria:** `cargo build --no-default-features` (or without `jit`) has no JIT-related `unsafe`.

---

### Phase 4 — Hardening & docs (1 day) — **P3**

1. Recreate short `docs/unsafe-code-guide.md` for contributors (link to this audit).
2. Property/fuzz tests for TypedArray and SharedValue/`Rc` ropes.
3. `cargo geiger` baseline in CI (informational).
4. Review all `unsafe impl Send/Sync` with explicit single-thread or ownership proofs.

---

## 6. What will **never** be fully “always safe” in-tree

If the product keeps these features, **some** `unsafe` must remain (in our code or in dependencies):

1. **C FFI exports** — embedding Tails in C/other languages.
2. **Native modules** — loading user `.so` and calling function pointers.
3. **JIT** — executable memory + calling generated code.
4. **Freeing foreign C strings** — `CString::from_raw` when ownership was transferred.

“Always safe” at the **application API** level (TypeScript/JS user code cannot trigger UB without native modules / bugs in runtime) is achievable by:

- confining `unsafe` to audited modules,
- not exposing raw pointers to TS,
- validating all lengths and UTF-8 at boundaries.

“Always safe” as **zero `unsafe` keywords in the repo** is **not realistic** for a JS/TS runtime with native modules and optional JIT.

---

## 7. Recommended success metrics

| Metric | Current (approx.) | After Phase 1 | After Phase 2–3 |
|--------|-------------------|---------------|-----------------|
| Non-wrapper production `unsafe` blocks | ~36 | ~25 | ~15–20 (FFI+JIT+loader only) |
| `transmute` of function pointers | 2 (calls + JIT entry) | 1 (JIT only) | 1 or 0 if JIT off |
| Raw `libc::` in modules | 7 | 0 | 0 |
| TypedArray unsafe | 2 | 0 | 0 |
| SharedValue unsafe | 2 | 0 | 0 |
| Documented Safety comments coverage | Partial | Full on new/changed | Full allowlist |

---

## 8. Suggested PR breakdown

1. **PR1:** TypedArray safe read/write + tests.  
2. **PR2:** SharedValue → `Rc` (or refcount fix) + rope tests.  
3. **PR3:** `nix` for os/process syscalls.  
4. **PR4:** Typed `NativeFn` storage; remove call-site transmute.  
5. **PR5:** Finish Safe* migration in ffi + abi; unify loaders.  
6. **PR6:** Feature-gate simd-json and JIT; CI allowlist.  
7. **PR7:** Docs + geiger baseline.

---

## 9. Decision guide (for future reviews)

```
Is this code talking to C / OS / executable memory?
  YES → unsafe allowed only in allowlisted modules; wrap for callers.
  NO  → can we use std / safe crate?
          YES → remove unsafe.
          NO  → write justification in this doc + Safety comment.
```

---

## 10. Conclusion

| Question | Answer |
|----------|--------|
| Can we replace **all** `unsafe` with always-safe code? | **No**, not while keeping FFI, native modules, and JIT. |
| Can we make **most** of the runtime free of local `unsafe`? | **Yes.** Phases 1–2 remove or encapsulate the avoidable majority. |
| Why does `unsafe` remain? | Language/OS boundaries: raw pointers, dynamic symbols, executable pages, C strings. |
| Highest-value fixes now? | TypedArray, SharedValue/`Rc`, nix for libc, typed native fns (no transmute). |

This plan prioritizes **real safety and correctness** over a vanity “0 unsafe lines” metric, while still driving scattered `unsafe` toward a small, audited surface.
