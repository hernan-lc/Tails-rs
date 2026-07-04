# Roadmap

> Audit-and-optimize roadmap. Focus: correctness review, hot-path profiling, targeted micro-optimisations.
> Completed phases (Pass 1 / 2 / 2a / 2b) are in `CHANGELOG.md`.

---

## ✅ Completed

| Phase | Item | Summary |
|-------|------|---------|
| 0.1 | GC `heap_value_to_index` | All `Value::X` variants now traced |
| 0.2 | Property access miss | Allocation-free `find_accessor` scan |
| 1.1 | `find_accessor` | Per-miss allocation removed |
| 1.2 | `heap_value_to_index` variants | Correct GC trace |
| 1.3 | Inline `LoadConst` BigInt/Symbol | Function call removed |
| 1.4 | `SuspendedFrame` `mem::take` | Stack snapshot optimized |
| 1.5 | `Vec::with_capacity` snapshots | Reallocation churn removed |
| 1.6 | `Value::String` → `Arc<str>` | 928 tests pass, zero failures |
| 1.7 | `ConsString` rope | O(1) concat, lazy flatten |
| 1.8 | Closure env cache | Per-sibling clone eliminated |
| 2.2 | GC nursery boundary + write-barrier scaffold | `nursery_start/next` tracking on `sweep`; `write_barrier()` hook placed for follow-up minor-GC |
| 2.3 | Inline `to_string_coerce` in `add` | 4 dedicated arms added |
| 2.4 | `JsIterator` heap type | Property insert removed |
| 2.5 | `Vec::with_capacity` native fns | Growth reallocs eliminated |
| 2.6 | Inline property storage (`PropertyStorage`) | Inline `[Option<(String, Value)>; 8]` for ≤8 prop objects, `FxHashMap` fallback. 547 tests pass |

---

## 🔄 In Progress / Next Up

### Phase 2 — Dispatch and GC (large scope)

- [ ] **2.1 — `Value` NaN-boxing**  
  Shrink 32-byte `Value` enum to 8 bytes via tagged-NaN float + `Box<HeapValue>`.  
  Touches every `match Value::X` in codebase. Expected: 2–4× general speedup.

---

## 📋 Out of Scope

- New feature work (modules, globals, APIs)
- Public release packaging (npm, docs site)
- Platform expansion (Windows FFI)
- Unsafe-code / memory-leak audit (on v1.0.0 list)

---

## 📚 References

- Pre-existing phases: `CHANGELOG.md`
- Benchmarks: `benchmarks/runner.sh`, `benchmarks/fixtures/`
- Regression tests: `src/vm/gc.rs::tests`, `tests/phase2_features.rs`, `src/vm/interpreter/mod.rs::execute_from`
