# Tails-rs Performance Optimization Plan

## Completed Optimizations

### Phase 1: Fix String Concatenation Performance
**Commit:** `c4d88bc` / `56a6e0b`
- ConsString clone from O(N) deep-tree to O(1) via SharedValue (raw pointer)
- `new_smart` heuristic: eagerly flatten short strings (<32 chars each)
- `std::mem::replace` pattern on AddLocal hot path to avoid clones
- **Result:** string_concat TIMEOUT → 39ms (vs Bun 2.7ms, 14.4x gap)

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
- **Result:** map_set 1040ms → 926ms (11% improvement)

### Phase 5: Lazy Prototype Allocation
**Commit:** `7b877a3`
- Functions no longer allocate a prototype object at creation time
- Prototype only created on demand when "prototype" property is accessed
- **Result:** eliminates 1 heap allocation per function creation

### Phase 6a: Inline Hot Instructions in Dispatch Loop
**Commit:** `a5deac3`
- Inlined Add, Sub, Eq, GetProperty, SetProperty directly in main match
- SetProperty includes fast path for common Object property assignment
- **Result:** eliminates function call overhead for most frequent instructions

### Phase 6b: Optimize find_accessor with has_accessors Flag
**Commit:** `7cd0933`
- PropertyStorage tracks whether any getter/setter accessors exist
- Skips O(N) linear scan in find_accessor when no accessors present
- SetProperty skips setter_key allocation when no accessors exist
- **Result:** faster property access for objects without accessors

## Final Benchmark Results (Tails vs Bun)

| Benchmark | Tails (ms) | Bun (ms) | Ratio |
|-----------|-----------|---------|-------|
| async_await | 32 | 5.3 | 6x |
| promises | 2014 | 27 | 74.6x |
| array_push | 82 | 6.7 | 12.2x |
| date | 522 | 31.3 | 16.7x |
| json_parse | 746 | 119 | 6.3x |
| map_set | 926 | 16.7 | 55.5x |
| promise_chain | 156 | 4 | 39x |
| regexp | 2089 | 100 | 20.9x |
| string_concat | 39 | 2.7 | 14.4x |
| generators | 665 | 12 | 55.4x |
| loops | 1802 | 8.3 | 217x |
| oo | 1461 | 26.7 | 54.7x |
| closures | TIMEOUT | 36 | ∞ |

## Remaining Optimization Roadmap

### Short-term (achievable)
- **Inline caches for property access**: Cache property indices to avoid repeated string comparisons
- **Shape/hidden classes**: Assign shapes to objects for O(1) property lookup
- **Fused increment+compare**: Combine loop counter increment and comparison into single instruction
- **String interning**: Deduplicate string values to reduce memory and comparison cost

### Medium-term
- **Promise inlining**: Inline promise resolution to reduce state machine overhead
- **Arena allocation**: Use bump allocation for short-lived objects to reduce GC pressure
- **Optimize GC root passing**: Pass stack/call_stack by reference to avoid cloning

### Long-term
- **Baseline JIT**: Compile hot loops to native code (10-50x improvement for compute-bound benchmarks)
- **Generational GC**: Young-gen only collection for short-lived objects
- **Hidden classes + inline caches**: V8-style property access optimization
