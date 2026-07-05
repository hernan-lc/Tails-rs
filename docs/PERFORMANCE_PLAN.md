# Tails-rs Performance Optimization Plan

## Completed Optimizations

### Phase 1: Fix String Concatenation Performance
**Commit:** `c4d88bc` / `56a6e0b`
- ConsString clone from O(N) deep-tree to O(1) via SharedValue (raw pointer)
- `new_smart` heuristic: eagerly flatten short strings (<32 chars each)
- `std::mem::replace` pattern on AddLocal hot path to avoid clones
- **Result:** string_concat TIMEOUT → 36ms (vs Bun 2.7ms, 13.4x gap)

### Phase 2: Lazy Closure Environment
**Commit:** `162b4b6`
- `shared_closure_env` changed from `HashMap::new()` to `Option<HashMap>`
- Only allocated when MakeClosure instruction needs it (rare)
- **Result:** eliminates HashMap alloc per function call

### Phase 3: Fix console.log + Benchmark Runner
**Commit:** `bcb7dae`
- Fixed color closure returning empty string when colors enabled
- Switched to `tails_process::stdout_write` with explicit flush
- Fixed benchmark runner to filter `[tails]` header
- **Result:** benchmark runner now works correctly

### Phase 4: Map/Set Native Bytecodes
**Commit:** `5d4ee4e`
- Added MapGet, MapSet, MapHas, MapDelete, SetAdd, SetHas, SetDelete bytecodes
- Compiler detects method call patterns and emits fast-path bytecodes
- VM handlers bypass string property lookup and native function dispatch
- **Result:** map_set improved

### Phase 5: Lazy Prototype Allocation
**Commit:** `7b877a3`
- Functions no longer allocate a prototype object at creation time
- Prototype only created on demand when "prototype" property is accessed
- **Result:** eliminates 1 heap allocation per function creation

## Final Benchmark Results (Tails vs Bun)

| Benchmark | Tails (ms) | Bun (ms) | Ratio |
|-----------|-----------|---------|-------|
| async_await | 27 | 5.3 | 5.1x |
| promises | 1669 | 27 | 62x |
| array_push | 72 | 6.7 | 10.8x |
| date | 526 | 31.3 | 16.8x |
| json_parse | 606 | 119 | 5.1x |
| map_set | 1347 | 16.7 | 80.8x |
| promise_chain | 111 | 4 | 27.7x |
| regexp | 2202 | 100 | 22x |
| string_concat | 36 | 2.7 | 13.4x |
| generators | 600 | 12 | 50x |
| loops | 1858 | 8.3 | 223x |
| oo | 1360 | 26.7 | 51x |
| closures | TIMEOUT | 36 | ∞ |

## Remaining Optimization Roadmap

### Short-term
- **Inline caches for property access**: Cache property indices to avoid repeated string comparisons
- **Shape/hidden classes**: Assign shapes to objects for O(1) property lookup
- **Fused increment+compare**: Combine loop counter increment and comparison into single instruction

### Medium-term
- **String interning**: Deduplicate string values to reduce memory and comparison cost
- **Promise inlining**: Inline promise resolution to reduce state machine overhead
- **Arena allocation**: Use bump allocation for short-lived objects to reduce GC pressure

### Long-term
- **Baseline JIT**: Compile hot loops to native code (10-50x improvement for compute-bound benchmarks)
- **Generational GC**: Young-gen only collection for short-lived objects
- **Hidden classes + inline caches**: V8-style property access optimization
