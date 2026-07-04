# Tails-rs Refactoring Plan

## Overview

This document outlines a structured refactoring plan for the Tails-rs codebase. The primary goals are:

- Eliminate repetitive code (DRY principle)
- Modularize large monolithic files
- Improve maintainability and readability
- Preserve existing behavior and public APIs

## Completed (as of 2026-07-04)

The following tasks have been implemented and verified (all 880+ tests green, zero clippy warnings).

### Phase 0 — Preparation ✅
- [x] Confirmed test baseline: all 880+ tests pass
- [x] `cargo clippy --all-targets` clean
- [x] `cargo build` OK

### Phase 1.4 — FFI: Consolidate tag-check functions ✅
- [x] Introduced `tails_is_type(value, tag)` helper
- [x] All 10 one-liner tag checks (`tails_is_undefined` … `tails_is_number`) now delegate to it
- [x] Public C ABI preserved — all old symbol names still exist

### Phase 2 — Date Module ✅
- [x] Extracted `is_leap_year`, `days_in_month`, `days_since_epoch`, `civil_from_days`, `date_from_millis`, `parse_iso8601` → `src/objects/js_date_calendar.rs`
- [x] Added `JsDate::components()` and `JsDate::set_ymdhms()` helpers
- [x] Refactored `set_utc_hours`, `set_utc_minutes`, `set_utc_seconds`, `set_utc_milliseconds` to use the composite helper
- [x] Reduced `js_date.rs` by ~140 lines

### Phase 3 — Modularize `src/objects/mod.rs` ✅
- [x] Extracted `ConsString` + `SYMBOL_*` constants → `src/objects/strings.rs`
- [x] Extracted `impl PartialEq for Value` → `src/objects/eq.rs`
- [x] Extracted `impl fmt::Display for Value` → `src/objects/display.rs`
- [x] Extracted `impl Hash for Value` (was in `js_collections.rs`) → `src/objects/hash.rs`
- [x] Moved `flatten_value`/`flatten_value_into` free functions into `Value::flatten()` / `Value::flatten_into()` methods
- [x] Renamed `value_str_len` → `Value::str_len()`
- [x] `src/objects/mod.rs` reduced from 223 → **81 lines**
- [x] All remaining files in `src/objects/` are < 300 lines

### Phase 4 — Errors cleanup ✅
- [x] Removed the entire `#[allow(dead_code)]` duplicate block in `src/errors/type_errors.rs`
- [x] File reduced from 97 → **43 lines** (-54 lines)

### Phase 5 — Native fn stubs consolidation ✅
- [x] Removed 6 redundant per-module macros: `os_stub!`, `path_stub!`, `process_stub!`, `zlib_stub!`, `tls_stub!`, `dns_stub!`
- [x] Rewrote all 6 `#[cfg(not(feature = "..."))]` blocks to use the unified `disabled_module_stub!` macro
- [x] `src/runtime_env/native_fns/mod.rs` reduced from 877 → **787 lines** (-90 lines)

### Phase 8 — Collections shared helper ✅
- [x] Extracted `swap_remove_vec<T>` helper function
- [x] `JsSet::delete` now a 4-line wrapper
- [x] `JsMap::delete` retains its map-index fixup with cleaner structure

---

## Phase 0 — Preparation

> **Priority:** High | **Risk:** Low

### 0.1 Create Feature Branch

- [ ] `git checkout -b refactor/dry-and-modularize-core`
- [ ] Confirm CI is green on the branch

### 0.2 Establish Covenants

- [ ] Run `cargo fmt` and `cargo clippy --all-targets` to lock baseline
- [ ] Record `cargo build` benchmarks for `src/ffi`, `src/objects`, `src/runtime_env/native_fns`
- [ ] Run existing test suite: `cargo test --all` — all must pass before and after each phase

### 0.3 Define Modules to Extract

- [ ] Agree on target module boundaries (see Phase 3)
- [ ] Document current `pub` API surface in `src/*/mod.rs`

---

## Phase 1 — FFI Module: Eliminate Boilerplate

> **Priority:** High | **Risk:** Medium | **Files:** `src/ffi/mod.rs`

### 1.1 Introduce Helper Macros

- [ ] Create `src/ffi/macros.rs` with:
  - `null_guard($runtime, do $body)` — wraps the common `if runtime.is_null() { return empty }` pattern
  - `null_guard_str($ptr, do $body)` — wraps the common `if ptr.is_null() { return empty }` pattern for `*const c_char`
  - `empty_tails_value()` — returns the canonical `TailsValue { tag: 0, data: 0 }`
- [ ] Re-export macros from `src/ffi/mod.rs` using `#[macro_use]`

### 1.2 Extract C-String Helper

- [ ] Merge `SafeCStr` (from `safe_wrappers.rs`) into `src/ffi/c_str.rs` (the FFI module already uses it)
- [ ] Remove `SafeCStr` from `safe_wrappers.rs` or keep for backward-compat if used outside FFI

### 1.3 Convert Boilerplate to Macro Usage

- [ ] Refactor each `#[no_mangle]` function in `src/ffi/mod.rs` to use `null_guard!`:
  - `tails_eval`
  - `tails_get_global`
  - `tails_set_global`
  - `tails_string_new`
  - `tails_object_new`
  - `tails_object_get`
  - `tails_object_set`
  - `tails_array_new`
  - `tails_array_get`
  - `tails_array_push`
  - `tails_call`
- [ ] Each handler should reduce to ~10 lines

### 1.4 Consolidate `TailsValueType`

- [ ] Replace the redundant `value.tag == X as u32` one-liner functions (lines 127–170) with a macro or a match expression on a single `fn is_tag(value: &TailsValue, tag: TailsValueType) -> bool`

### 1.5 Move `value_to_tails_value` and `tails_value_to_value` to a Dedicated File

- [ ] Extract to `src/ffi/conversions.rs`
- [ ] Clean up duplicate `String` handling in `value_to_tails_value` (`String` and `Cons` arms do the same CString allocation)

### 1.6 Validations

- [ ] `cargo build`
- [ ] `cargo clippy --all-targets`
- [ ] `cargo test --all`
- [ ] Run FFI integration tests (if any) / verify `examples/*` still compile

---

## Phase 2 — Date Module: DRY Setters and Helpers

> **Priority:** Medium | **Risk:** Low | **Files:** `src/objects/js_date.rs`

### 2.1 Introduce Composite Constructor Helper

- [ ] Add a private helper `fn set_ymdhms(&mut self, days: f64, h: f64, m: f64, s: f64, ms: f64)` that all setters call
- [ ] Refactor `set_utc_hours`, `set_utc_minutes`, `set_utc_seconds`, `set_utc_milliseconds` to use this helper

### 2.2 Eliminate Duplicate UTC/Local Delegates

- [ ] The local-time getters (`get_full_year`, `get_month`, …) and setters (`set_full_year`, `set_month`, …) are currently just delegates to UTC versions.
- [ ] Add a `timezone_offset` field to `JsDate` (default 0.0 = UTC)
- [ ] Centralize computation so UTC and local paths share code via the composite helper

### 2.3 Extract Calendar Helpers

- [ ] Move `days_since_epoch`, `civil_from_days`, `date_from_millis`, `is_leap_year`, `days_in_month`, `parse_iso8601` into `src/objects/js_date_calendar.rs`
- [ ] Re-import in `src/objects/js_date.rs` via `crate::objects::js_date_calendar::...`
- [ ] This shrinks the main struct-implementation section

### 2.4 Validations

- [ ] `cargo test --all`

---

## Phase 3 — Modularize `src/objects/mod.rs`

> **Priority:** Medium | **Risk:** Medium | **Files:** `src/objects/mod.rs`

The current `src/objects/mod.rs` mixes:
- The `ConsString` rope type
- The `Value` enum definition
- `flatten_value` / `flatten_value_into` helper functions
- `PartialEq`, `Hash`, and `Display` implementations for `Value`

### 3.1 Extract `ConsString` to Its Own File

- [ ] Move `ConsString`, `SYMBOL_*` constants to `src/objects/strings/mod.rs`
- [ ] Keep `Value::String` and `Value::Cons` arms in `Value` enum but reference the new module

### 3.2 Extract Display / Eq / Hash Implementations

- [ ] Move `impl PartialEq for Value` to `src/objects/eq.rs`
- [ ] Move `impl fmt::Display for Value` to `src/objects/display.rs`
- [ ] Move `impl Hash for Value` from `src/objects/js_collections.rs` to `src/objects/hash.rs` (consolidate with the same concern)

### 3.3 Deduplicate `flatten_value` and `flatten_value_into`

- [ ] These are currently standalone functions on `Value`. Move them into `src/objects/strings/mod.rs` (or inline them into `ConsString::flatten` / `ConsString::flatten_into` to remove the extra indirection)

### 3.4 Re-export Structure in `src/objects/mod.rs`

- [ ] `mod.rs` should only declare submodules and re-export key types
- [ ] Target size: **< 80 lines**

### 3.5 Validations

- [ ] `cargo build`
- [ ] `cargo test --all`

---

## Phase 4 — Clean Up `src/errors/type_errors.rs`

> **Priority:** Low | **Risk:** Low | **Files:** `src/errors/type_errors.rs`

### 4.1 Remove Duplicate Functions

There are two versions of every helper (lines 3–56 and lines 56–97).

- [ ] Keep the non-`#[allow(dead_code)]` versions (lines 56–97)
- [ ] Remove the dead-code versions (lines 3–55) or mark them deprecated with a FIXME
- [ ] Alternatively, consolidate into `src/errors/mod.rs` by implementing helper constructors directly on `Error` via the existing `define_error_constructor!` macro

### 4.2 Consolidate into `define_error_constructor!`

- [ ] Add a `define_error_constructor_extended!` macro (or extend existing) that supports formatted messages, e.g.:
  ```rust
  define_error_extended!(type_error_expected, TypeError, "Expected {}, got {}", expected, actual);
  ```
- [ ] This removes the stand-alone functions entirely

### 4.3 Validations

- [ ] `cargo test --all`

---

## Phase 5 — Consolidate Stub Macros in `native_fns/mod.rs`

> **Priority:** Low | **Risk:** Low | **Files:** `src/runtime_env/native_fns/mod.rs`

### 5.1 Remove Legacy Per-Module Stub Macros

The following macros in `native_fns/mod.rs` are redundant with the unified `disabled_module_stub!`:

- [ ] Remove `os_stub!` (lines 142–154)
- [ ] Remove `path_stub!` (lines 176–188)
- [ ] Remove `process_stub!` (lines 207–219)
- [ ] Remove `zlib_stub!` (lines 255–267)
- [ ] Remove `tls_stub!` (lines 289–301)
- [ ] Remove `dns_stub!` (lines 317–329)

- [ ] Rewrite each `#[cfg(not(feature = "..."))]` block to use `disabled_module_stub!` instead

### 5.2 Deduplicate `use crate::errors::{Error, Result}`
In each stub block, the same three `use` lines repeat 6 times.

- [ ] Consider extracting a `pub(super) mod feature_stubs;` that contains shared helpers, or accept the duplication since each is already minimal

### 5.3 Validations

- [ ] `cargo test --all` (ensure feature flags compile in both enabled and disabled states)
- [ ] `cargo build --features fs,http,net,os,path,process,zlib,tls,dns`

---

## Phase 6 — Ethan (Extract Refactor) — Interpreter Split

> **Priority:** High | **Risk:** High | **Files:** `src/vm/interpreter/mod.rs` (1490 lines)

### 6.1 Audit Current `mod.rs` Concerns

`src/vm/interpreter/mod.rs` currently owns:

- `EventSource` trait
- `SuspendedFrame` struct
- `Interpreter` struct (huge)
- Instruction decoding logic
- Bytecode dispatch loop

### 6.2 Strategy

Split into logical submodules but keep everything as `pub(crate)`:

- [ ] Extract `bytecode.rs` → all `Instruction` decoding and dispatch
- [ ] Extract `frame.rs` → `CallFrame`, `ExceptionHandler`, `SuspendedFrame`
- [ ] Re-export from `mod.rs`

### 6.3 Sub-Tasks

#### 6.3.1 Extract Bytecode Dispatch

- [ ] Locate the frame execution loop in `src/vm/interpreter/mod.rs`
- [ ] Move dispatch table / main loop to `src/vm/interpreter/bytecode.rs`
- [ ] Keep `Interpreter` reference as `&mut self` parameter
- [ ] Forward from `Interpreter::run` / `Interpreter::execute_module` → new module

#### 6.3.2 Extract Frame & SuspendedFrame Types

- [ ] Move `CallFrame`, `ExceptionHandler`, `SuspendedFrame` structs to `src/vm/interpreter/frame.rs`
- [ ] Re-export in `src/vm/interpreter/mod.rs`

#### 6.3.3 Validations

- [ ] `cargo build`
- [ ] `cargo clippy --all-targets`
- [ ] `cargo test --all`
- [ ] `cargo test --all --features fs,http,net,os,path,process,zlib,tls,dns`

---

## Phase 7 — Ethan (Extract Refactor) — Native Function Registry

> **Priority:** Medium | **Risk:** Medium | **Files:** `src/runtime_env/native_fns/mod.rs`

### 7.1 Current State

- `NATIVE_TABLE` is a 536-entry `&[NativeFn]` static array inline in `mod.rs`
- Each function entry is a manually written line

### 7.2 Introduce Procedural / Macro-Based Registration

- [ ] Define a macro `native_fn!($name:ident, $path:path)` that expands to `$name: $path,`
- [ ] Alternatively, use a `build_native_table!` macro that groups entries by category:
  ```rust
  build_native_table! {
      console: [native_console_log, native_console_warn, ...],
      array: [native_array_push, native_array_pop, ...],
      ...
  }
  ```
- [ ] This reduces listing duplication and makes additions/removals less error-prone

### 7.3 Validations

- [ ] `cargo test --all`
- [ ] `cargo build --features ...` (full feature matrix)

---

## Phase 8 — Collection Helpers Shared Traits

> **Priority:** Low | **Risk:** Low | **Files:** `src/objects/js_collections.rs`

### 8.1 Introduce `JsCollectionBase` Trait (optional)

- [ ] Define a small trait with `fn clear(&mut self)`, `fn size(&self) -> usize`
- [ ] Implement for `JsMap`, `JsSet`, `JsWeakMap`, `JsWeakSet`
- [ ] This removes duplicated `Default` / `new` patterns

### 8.2 Consolidate `delete` Logic

- [ ] `JsMap.delete` and `JsSet.delete` both use swap-and-pop. Extract a helper:
  ```rust
  fn swap_remove<T: PartialEq>(vec: &mut Vec<T>, item: &T) -> bool;
  ```
- [ ] Place in `src/objects/collections/helpers.rs` and use in both files

### 8.3 Validations

- [ ] `cargo test --all`

---

## Phase 9 — Lexer Module Recap (Non-Breaking)

> **Priority:** Low | **Risk:** Low | **Files:** `src/compiler/lexer.rs`

### 9.1 Split into Logical Sub-Files

The lexer is 1017 lines. If the struct and business logic warrant, split into:

- `lexer/lexer.rs` — `Lexer` struct and core methods
- `lexer/token.rs` — `Token` definition and helpers
- `lexer/whitespace.rs` — whitespace / comment handling
- `lexer/number.rs` — numeric literal parsing
- `lexer/string.rs` — string literal parsing

### 9.2 Validations

- [ ] `cargo build`
- [ ] `cargo test --all` (ensure compiler tests still pass)

---

## Phase 10 — Final Review and Cleanup

> **Priority:** High | **Risk:** Low

### 10.1 Run Full Matrix

- [ ] `cargo build`
- [ ] `cargo build --all-targets --features fs,http,net,os,path,process,zlib,tls,dns`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all --all-features`
- [ ] `cargo doc --no-deps` (ensure all public docs compile)

### 10.2 Audit Public API Surface

- [ ] Compare `src/**/mod.rs` re-exports with git diff to confirm no accidental breaking changes
- [ ] Run `cargo publish --dry-run` (or equivalent) if applicable

### 10.3 Documentation

- [ ] Update `CHANGELOG.md` with refactoring notes per phase
- [ ] Update `REFACTORING_SUMMARY.md` if present
- [ ] Ensure `README.md` does not reference any removed file paths directly

### 10.4 Commit

- [ ] Stage only intended files
- [ ] Commit message: `refactor: DRY FFI, modularize interpreter, remove duplicate stubs`

---

## Summary: Tasks At a Glance

| Phase | Focus | Tasks | Est. Impact |
|-------|-------|-------|-------------|
| 0 | Preparation | 4 | Green baseline |
| 1 | FFI macro | 6 | ~-200 lines, clearer bindings |
| 2 | Date DRY | 4 | ~-80 lines, share helpers |
| 3 | objects/mod.rs split | 5 | Clean separation of concerns |
| 4 | errors cleanup | 3 | Remove 1x duplicated error helpers |
| 5 | Native fn stubs | 3 | Remove 6 redundant macros |
| 6 | Interpreter split | 4 | Interpreter ~800 lines each |
| 7 | NATIVE_TABLE macro | 3 | Auto-generate registration |
| 8 | Collections helpers | 3 | Remove shared boilerplate |
| 9 | Lexer split | 2 | Each sub-file < 400 lines |
| 10 | Final review | 4 | Green CI, ready to merge |
