# Unsafe Code Guide (Tails-rs)

This is the short maintainer guide for writing and reviewing `unsafe` in this
repository. For the full inventory, phased plan, and site-by-site decisions, see
[`UNSAFE_AUDIT_PLAN.md`](./UNSAFE_AUDIT_PLAN.md).

## Policy

1. **Prefer safe APIs** — `std`, `nix`, byte-oriented TypedArray helpers, typed
   `NativeFn` storage — before writing new `unsafe`.
2. **Confine** remaining `unsafe` to allowlisted modules (see below).
3. **Document** every `unsafe` block / `unsafe fn` with `# Safety` or
   `// Safety:` explaining the invariant the caller upholds.
4. **`unsafe_op_in_unsafe_fn`** is warned crate-wide: even inside `unsafe fn`,
   wrap unsafe operations in an inner `unsafe { ... }` block.
5. Do **not** scatter raw pointer deref / `transmute` / `libc` calls in
   interpreter opcodes, builtins, or business logic.

## Allowlisted locations (inherent or encapsulating)

| Area | Paths | Why |
|------|--------|-----|
| Safe wrappers | `src/ffi/safe_*.rs`, `src/vm/interpreter/safe_*.rs` | Encapsulate FFI pointers |
| C FFI surface | `src/ffi/mod.rs`, `src/ffi/native.rs`, `src/ffi/conversions.rs` | C ABI boundary |
| Native ABI | `modules/abi/**`, `modules/native-macros/**` | Module ABI / macros |
| Dynamic load | `modules.rs` init path | `libloading` + module handle |
| JIT | `src/vm/jit/**` (feature `jit`) | Executable memory |
| Optional fast JSON | `json_fns.rs` when `fast-json` | `simd_json::from_str` |

## Safe patterns already in tree

- **`SafePtr` / `SafeCStr` / `SafeSlice` / `SafeFFIString`** — use at every C
  boundary entry after null checks.
- **`SafeLibrary`** — load `.so`/`.dylib`/`.dll`; keep the library alive for
  symbol lifetime (or `mem::forget` when process-lifetime is intentional).
- **Typed `Vec<tails_abi::NativeFn>`** — no `transmute` at call sites.
- **TypedArray `NeBytes`** — bounds-checked `from_ne_bytes` / `to_ne_bytes`.
- **`Arc<Value>` rope children** — no custom raw-pointer refcount.
- **`nix`** — Unix `kill`, uid/gid, hostname without local `libc` unsafe.

## Features related to unsafe surface

| Feature | Default | Effect |
|---------|---------|--------|
| `jit` | on | Enables JIT module (mmap / RX pages). Disable to drop JIT unsafe. |
| `fast-json` | off | Uses unsafe simd-json parse path; default is safe `serde_json`. |

## Checklist for new `unsafe`

- [ ] Can this be done with a safe crate or existing wrapper?
- [ ] Is the code in an allowlisted module?
- [ ] Is there a `# Safety` section on the `fn` and/or `// Safety:` on the block?
- [ ] Are preconditions checked (null, length, UTF-8) before the unsafe op?
- [ ] Did you run `scripts/check_unsafe_allowlist.sh`?

## Measuring

```bash
./scripts/check_unsafe_allowlist.sh
# optional: cargo install cargo-geiger && cargo geiger
```
