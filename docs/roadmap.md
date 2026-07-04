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
| 2.3 | Inline `to_string_coerce` in `add` | 4 dedicated arms added |
| 2.4 | `JsIterator` heap type | Property insert removed |
| 2.5 | `Vec::with_capacity` native fns | Growth reallocs eliminated |

---

## 🔄 In Progress / Next Up

### Phase 2 — Dispatch and GC (large scope)

- [ ] **2.1 — `Value` NaN-boxing**  
  Shrink 32-byte `Value` enum to 8 bytes via tagged-NaN float + `Box<HeapValue>`.  
  Touches every `match Value::X` in codebase. Expected: 2–4× general speedup.

- [ ] **2.2 — Generational / bump allocator**  
  Replace mark-sweep with bump allocator (young gen) + small mark-sweep (old gen).  
  Current `Vec<HeapValue>::clone()` on `pc & 127 == 0` is the top per-N-instruction cost.

---

### Phase 3 — Profile-guided follow-ups

After Phase 2 lands, re-run `benchmarks/runner.sh` and pick top-3 hotspots.

- [ ] **3.4 — RegExp direct fast-path + lazy result**  
  Next-largest gap on `builtins/regexp.js` after iterator fast-path.

- [ ] **3.5 — Inline property storage for small objects**  
  Replace `FxHashMap<String, Value>` with `[Option<(Rc<str>, Value)>; 8]` for ≤8-property objects.  
  Expected: 5–10× on `core/oo.js`.

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