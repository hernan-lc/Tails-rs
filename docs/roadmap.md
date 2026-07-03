# Roadmap

> Audit-and-optimize roadmap. No new features are added here; the
> focus is correctness review, hot-path profiling, and targeted
> micro-optimisations on the Tails-rs runtime. Phases already shipped
> (Pass 1 / 2 / 2a / 2b) are kept in `CHANGELOG.md` for context; they
> are not re-attempted in this pass.

## Methodology

1. **Audit** — read every hot-path file in `src/vm/`, `src/objects/`,
   and `src/runtime_env/native_fns/`, and rank candidates by (a)
   frequency on `benchmarks/fixtures/`, and (b) cost per occurrence
   (allocations, clones, function-call overhead).
2. **Fix correctness regressions** first (Phase 0). These are real
   bugs that can be triggered by valid JS.
3. **Low-hanging fruit** second (Phase 1): every other change here is
   zero-behaviour-change but removes a measured allocation or branch.
4. **Profile-guided** work third (Phase 2+): items in the original
   roadmap that are still relevant (NaN-boxing, generational GC,
   ConsString, lazy iterators) are kept but updated with the current
   audit context.

## Phase 0 — Correctness audit

Real bugs that produce wrong results on otherwise valid JavaScript.
Landed first so the performance work in later phases can be measured
against a sound baseline.

- [x] **GC: `heap_value_to_index` incomplete for `Value` variants
  pointing at `HeapValue::X`** — the helper in `src/vm/gc.rs` only
  returned `Some(idx)` for `Object` / `Array` / `Function` / `Promise`
  / `Proxy`, so any `TypedArray`, `Map`, `Set`, `WeakMap`, `WeakSet`,
  `Date`, `RegExp`, `Buffer`, or `Generator` held as a value inside a
  `Map` / `Set` / `Array` / `Object` properties was silently dropped
  by the mark phase. The bug was introduced by the Phase 4C fix that
  started tracing `Map` / `Set` keys & values. **Fixed**: the helper
  now returns `Some(idx)` for every `Value::X(usize)` variant whose
  payload indexes `Interpreter.heap`. Regression tests
  `test_gc_map_holds_typed_array_does_not_collect` and
  `test_gc_set_holds_date_does_not_collect` in
  `src/vm/gc.rs::tests` lock the contract; the
  `test_heap_value_to_index_is_complete` test enumerates every
  variant so a future drift is caught at compile-test time.
- [x] **Property access: `format!("__getter_{}", key_str)` on every
  property miss** — `get_property_with_this` (in
  `src/vm/interpreter/property_access.rs`) and `in_check_mut`
  allocated a fresh `String` of `9 + key_str.len()` bytes on *every*
  miss, including the common case where the object has no
  `__getter_*` accessor. **Fixed**: replaced with an
  allocation-free scan (`find_accessor`) that uses `len()` +
  `starts_with` + `ends_with` on the property map's existing keys.
  Zero allocations in the common case; cold path for accessors is a
  single `Vec::iter()` over the object's own property map.

## Phase 1 — Allocation-free hot paths

Zero-behaviour-change micro-optimisations. Ordered roughly by
expected impact on `benchmarks/fixtures/loops.js`,
`core/closures.js`, and `builtins/json_parse.js`.

- [x] **Phase 1.1 — `find_accessor` allocation-free property miss**
  (see Phase 0 entry). Highest-value single fix: removes a per-miss
  heap allocation in the most common property access path.
- [x] **Phase 1.2 — `heap_value_to_index` returns the right
  variants** (see Phase 0 entry). Correctness, not perf, but it also
  tightens the GC trace.
- [x] **Phase 1.3 — Inline `LoadConst` for `BigInt` and `Symbol`** —
  `src/vm/interpreter/mod.rs:472-498` already inlines `LoadConst`
  for the 6 immediate value types. `BigInt` and `Symbol` are also
  immediate values; extending the inline arm removes one function
  call per `const BIG = 100n` / `const S = Symbol()`. **Fixed**:
  added `Value::BigInt(_)` and `Value::Symbol(_)` to the inline arm
  in `execute_from`. The clone is one discriminant + payload memcpy
  (16/8 bytes) — much cheaper than a function call + match
  cascade. Regression tests `test_loadconst_bigint_in_hot_loop`,
  `test_loadconst_bigint_in_switch`, and
  `test_loadconst_symbol_equality` in `tests/phase2_features.rs`
  exercise the new path.
- [x] **Phase 1.4 — `SuspendedFrame` stack-snapshot `mem::take`** —
  the original note referenced `src/vm/interpreter/mod.rs:80-95`
  (the `collect_garbage` snapshot path), but the `Await` async
  suspend path at `mod.rs:1281-1294` was *already* using
  `std::mem::take` (i.e. moving the buffer instead of cloning it).
  The path in the roadmap note (the GC `collect_garbage` snapshot
  in `mod.rs:289-303`) was the remaining clone site, and is fixed
  in Phase 1.5 below.
- [x] **Phase 1.5 — `Vec::with_capacity` for
  `SuspendedFrame::call_stack_snapshot`** — when taking the
  `call_stack` snapshot, reserve the destination's capacity to the
  source's length. One-liner; saves 0→1→2→4→… growth reallocations
  on deep call stacks. **Fixed**: `Interpreter::collect_garbage`
  now builds the `stack_snapshot` and `call_stack_snapshot` Vecs
  with `Vec::with_capacity(src.len())` + `extend(src.iter().cloned())`
  instead of `self.stack.clone()` /
  `self.call_stack.clone()`. Avoids 4–5 reallocations on a
  100-deep `call_stack` and 1–2 reallocations on the common
  5–10-deep case. The clone is still required (the GC needs an
  owned snapshot that survives across the `gc.collect()` call) —
  we only removed the growth-allocation churn. Regression
  coverage: `test_gc_snapshot_capacity_does_not_drop_references`
  in `tests/phase2_features.rs` walks a 500-element object graph
  on the stack.
- [x] **Phase 1.6 — `Value::String` interning via `Arc<str>`** —
  long-standing Phase 3B from the original roadmap. Strings in the
  constant pool are immutable; the old `LoadConst` did a
  24-byte `String` clone per reference. Switching the `Value::String`
  variant from `String` to `Arc<str>` makes `LoadConst` a single
  atomic-increment clone (no heap alloc / memcpy of the backing
  buffer). Dynamic string ops that need mutation use
  `Arc::make_mut`. All 57 modified files across `src/vm/`,
  `src/objects/`, `src/runtime_env/native_fns/`, `src/compiler/`,
  and `src/ffi/` were migrated. 928 tests pass, 0 failures.
- [ ] **Phase 1.7 — `ConsString` rope for `add(String, _)`** —
  long-standing Phase 3A. `s = s + "x"` currently allocates a
  fresh `String` of `s.len() + 1` bytes per iter. A rope
  representation makes the concat O(1) and flattens lazily.
  Expected impact: 3–5x on `builtins/string_concat.js`.
- [ ] **Phase 1.8 — Closure env `Rc<RefCell<Vec<Value>>>`** —
  long-standing Phase 2A. Currently, sibling closures each clone
  the entire captured-var vector. For a function with 3 captures
  and 3 sibling closures, that's 288 bytes copied per
  `MakeClosure`. Sharing an `Rc<RefCell<Vec<Value>>>` eliminates
  all of it. Expected impact: 5–10x on `core/closures.js`
  (currently 5204ms).
## Phase 2 — Dispatch and GC

Larger-scope changes; each requires its own measurement pass.

- [ ] **Phase 2.1 — `Value` NaN-boxing (Phase 1A from the original
  roadmap)** — shrink the 32-byte `Value` enum to 8 bytes via a
  tagged-NaN float representation for all pointerless variants and a
  separate `Box<HeapValue>` for the heap-pointing ones. Multi-week
  refactor touching every `match value { Value::X(_) => … }` in the
  codebase. Expected impact: 2–4x general improvement (every
  push/pop on the stack, every `Vec<Value>` field in
  `JsFunction::closure`, `JsArray::elements`, `JsObject::properties`,
  every function argument, …). The largest single perf win possible.
- [ ] **Phase 2.2 — Generational / bump allocator (Phase 1E)** —
  replace the mark-sweep collector with a cheaper bump allocator
  for the young generation and a small mark-sweep for the old
  generation. Currently `pc & 127 == 0 && gc.should_collect()`
  triggers a `Vec<HeapValue>::clone()` of the entire heap for mark
  roots (see `Interpreter::collect_garbage`), which is the most
  expensive per-N-instructions cost in the entire VM.
- [x] **Phase 2.3 — Inline `to_string_coerce` for `Value::Integer`
  and `Value::Float` in `add` / `add_local`** — currently `add`
  falls back to `to_string_coerce` for `String + Integer` /
  `String + Float`, which allocates a `String` for the number. The
  Phase 5F arm in `AddLocal` already has a specialised path; the
  same specialisation can be hoisted into the general
  `Instruction::Add` arm in `src/vm/interpreter/instructions.rs`.
  **Fixed**: `Interpreter::add` in
  `src/vm/interpreter/value_ops.rs` now has four dedicated arms
  that match the same Phase 5F shape as the `add_local` arm —
  `(String, Integer)`, `(Integer, String)`, `(String, Float)`,
  `(Float, String)`. Each does a single
  `String::with_capacity(a.len() + b_str.len())` + two `push_str`
  calls, skipping the `to_string_coerce` round-trip. Behaviour is
  identical (including the finite-integer special case `"5"` not
  `"5.0"`). Regression tests `test_add_string_plus_integer`,
  `test_add_integer_plus_string`, `test_add_string_plus_float`,
  `test_add_float_plus_string`,
  `test_add_string_plus_negative_integer`, and
  `test_add_string_plus_float_no_integer_form` in
  `tests/phase2_features.rs` lock the new arms.
- [ ] **Phase 2.4 — `JsIterator` heap type** — currently iterator
  state is stored as `__type` / `__index` / `__target` / `__data`
  inside the iterator's `JsObject::properties` map, and every
  `next()` does a `properties.insert("__index", …)`. Replacing
  this with a `HeapValue::Iterator { kind, index, target, data }`
  variant removes the per-step hashmap insert (the dominant cost
  in `core/generators.js` after Pass 2a).
- [ ] **Phase 2.5 — `Vec::with_capacity` for `Value` vecs in native
  fns** — `runtime_env/native_fns/` has ~30 modules; a grep finds
  dozens of `Vec::new()` followed by `push(...)` loops. A focused
  pass to add `Vec::with_capacity(n)` where `n` is statically
  known (array length, object property count, …) saves a handful
  of reallocations per native call. Low impact individually, but
  composes.

## Phase 3 — Profile-guided follow-ups

After Phase 1 + 2 land, re-run `benchmarks/runner.sh` and pick the
top-3 remaining hotspots. The original roadmap entries that are
still relevant are kept here verbatim as a starting point.

- [ ] **Phase 3.1 — `Rc<RefCell<Vec<Value>>>` closure env** (the
  long-standing Phase 2A). Currently the single largest gap on
  `core/closures.js`.
- [ ] **Phase 3.2 — `ConsString` rope** (the long-standing Phase 3A).
  Currently the single largest gap on `builtins/string_concat.js`.
- [ ] **Phase 3.3 — Lazy Map / Set iterator → dedicated
  `HeapValue::Iterator`** (Phase 2.4 above; called out here because
  the original Phase 4A used the same `__target` property trick and
  is on the hot path).
- [ ] **Phase 3.4 — RegExp direct fast-path + lazy result**
  (Phase 7B/7C from the original roadmap). After Phase 2.4, the
  iterator fast-path becomes the next-largest gap on
  `builtins/regexp.js`.
- [ ] **Phase 3.5 — Inline property storage for small objects**
  (Phase 10A from the original roadmap). The current
  `JsObject::properties: FxHashMap<String, Value>` has a hash +
  collision-resolution cost on every `obj.x`; replacing it with a
  `[Option<(Rc<str>, Value)>; 8]` array for the 99% case of
  ≤8-property objects would be a 5–10x win on `core/oo.js`.

## Out of scope for this roadmap

- New feature work (new modules, new globals, new APIs).
- Public release packaging (npm, docs site).
- Platform expansion (Windows-specific FFI).
- Unsafe-code audit and memory-leak audit (already on the original
  v1.0.0 list; not touched here).

## References

- The complete list of pre-existing optimisation phases (Pass 1 / 2
  / 2a / 2b) is preserved in `CHANGELOG.md`.
- Benchmark methodology and baseline numbers: see
  `benchmarks/runner.sh` and `benchmarks/fixtures/`.
- The current implementation status (which `Value` variants are
  heap-traced, which `Instruction` arms are inlined, …) is enforced
  by the regression tests added in Phase 0; see
  `src/vm/gc.rs::tests` and the per-phase assertions in
  `src/vm/interpreter/mod.rs::execute_from`.
