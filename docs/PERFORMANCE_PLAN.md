# Tails-rs Performance Optimization Plan

## Completed Optimizations

### Phase 1: Fix String Concatenation Performance
**Commit:** `c4d88bc` / `56a6e0b`
- ConsString clone from O(N) deep-tree to O(1) via SharedValue (raw pointer)
- `new_smart` heuristic: eagerly flatten short strings (<32 chars each)
- `std::mem::replace` pattern on AddLocal hot path to avoid clones
- **Result:** string_concat TIMEOUT → 29ms (vs Bun 2ms, 14.7x gap)

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
- Fallback to get_property + call_value for non-Map/Set objects
- **Result:** map_set 1040ms → 941ms (10% improvement)

## Current Benchmark Results (Tails vs Bun)

| Benchmark | Tails | Bun | Ratio |
|-----------|-------|-----|-------|
| string_concat | 29ms | 2ms | 14.7x |
| async_await | 27ms | 5ms | 5.7x |
| array_push | 69ms | 6ms | 12x |
| json_parse | 399ms | 91ms | 4.4x |
| date | 469ms | 29ms | 16x |
| promise_chain | 110ms | 3ms | 33x |
| generators | 537ms | 10ms | 55x |
| oo | 1317ms | 24ms | 54x |
| map_set | 941ms | 12ms | 76x |
| regexp | 1708ms | 93ms | 18x |
| loops | 1667ms | 8ms | 208x |
| promises | 1417ms | 26ms | 55x |
| closures | TIMEOUT | 24ms | ∞ |

## Remaining Optimization Roadmap

### Phase 5: Value Type Shrinking (string interning)
- Shrink Value from 24 bytes to 16 bytes via string interning
- Expected: 20-40% improvement across all benchmarks
- Status: Not started (invasive change)

### Phase 6: Loop and Dispatch Optimization
- Computed dispatch table for opcode dispatch (skip match arms)
- Fused increment+compare instructions for loops
- Expected: 30-50% improvement for loops benchmark
- Status: Not started

### Phase 7: Promise/Async/Generator Optimization
- Inline promise resolution, reduce state machine overhead
- Expected: 30-50% improvement for promises/generators
- Status: Not started

### Phase 8: JIT Compilation (long-term)
- Baseline JIT for hot loops
- Expected: 10-50x improvement for compute-bound benchmarks
- Status: Not started (massive effort)
