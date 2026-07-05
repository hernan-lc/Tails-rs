# Performance Optimization Plan — Phase 8

**Date:** 2026-07-05
**Baseline:** Previous REPORT.md (2026-07-05 03:46 UTC)
**Build:** `cargo build --release` (profile: optimized)

## Current Benchmark Results

| Benchmark | Tails (ms) | Bun (ms) | Node (ms) | Ratio (vs Bun) | Change from Prev |
|-----------|-----------|---------|----------|----------------|-----------------|
| loops | 1613 | 7.6 | 32 | 212x | 0% |
| map_set | 901 | 12.4 | 13 | 73x | 1% worse |
| regexp | 1568 | 95 | 78 | 17x | 13% better |
| generators | 524 | 9.4 | 10 | 56x | 9% better |
| promises | 1051 | 25 | 26 | 41x | 27% better |
| closures | TIMEOUT | 24 | 13 | ∞ | same |
| date | 470 | 29 | 177 | 16x | 3% better |
| json_parse | 388 | 92 | 115 | 4.2x | 30% better |
| array_push | 70 | 4.8 | 8 | 15x | 3% better |
| oo | 1025 | 27 | 87 | 38x | 25% better |
| promise_chain | 102 | 2.6 | 6 | 39x | 35% better |
| string_concat | 28 | 2 | 8 | 14x | 40% better |
| async_await | 26 | 5 | 9 | 5.2x | 4% better |
| fs_read_sync | 40 | 70 | 163 | 0.57x | 15% better |
| fs_write_sync | 8 | 5 | 11 | 1.6x | 11% better |

## Key Findings

### What improved since last baseline
- **promises**: 1437→1051ms (27%) — DeferredResolve/DeferredReject and microtask idle check paying off
- **json_parse**: 555→388ms (30%) — likely from JIT tick removal + general improvements
- **promise_chain**: 157→102ms (35%) — same promise optimizations
- **string_concat**: 47→28ms (40%) — ConsString optimizations maturing
- **oo**: 1360→1025ms (25%) — JIT tick removal removed a major regression
- **regexp**: 1805→1568ms (13%)

### What's still slow (ranked by absolute gap to Bun)

1. **loops (212x)** — 1613ms vs 7.6ms
   - 5M integer additions in a tight for-loop
   - Interpreter dispatch overhead dominates (~320ns per iteration)
   - Fused LoopBranch helps but can't match native dispatch

2. **map_set (73x)** — 901ms vs 12.4ms
   - 50K Map.set() + for-of iteration
   - Native Map bytecodes exist but still slow due to FxHashMap overhead

3. **generators (56x)** — 524ms vs 9.4ms
   - 2000 generator iterations × 100 yields each
   - Generator suspend/resume involves heap allocation per yield

4. **promises (41x)** — 1051ms vs 25ms
   - 100K promise creations + resolutions
   - Microtask queue drain overhead

5. **oo (38x)** — 1025ms vs 27ms
   - 100K object creations with constructor + method dispatch
   - Property access + prototype chain walking

6. **promise_chain (39x)** — 102ms vs 2.6ms
   - 500 chains × 20 .then() calls each
   - Each .then() creates a new promise + registers microtask

7. **closures (∞)** — TIMEOUT vs 24ms
   - 1M closures created and called
   - Complete hang suggests infinite loop or unbounded allocation

8. **regexp (17x)** — 1568ms vs 95ms
   - RegExp.exec in a while loop on large text
   - Regex engine likely interprets pattern each call

9. **date (16x)** — 470ms vs 29ms
   - 500K Date.now() + Date.parse() calls
   - Native Date implementation overhead

10. **array_push (15x)** — 70ms vs 4.8ms
    - 200K Array.push() calls
    - Dynamic array growth + Value cloning

## Optimization Roadmap

### Phase 8.1: Remove JIT tick from hot path ✅ DONE
- Removed `self.jit.tick()` call from LoopBranch handler
- The mutable borrow was preventing compiler optimizations in the dispatch loop
- **Result:** oo.js 6027ms→1025ms (6x improvement), other benchmarks neutral

### Phase 8.2: Inline caches for property access
**Target:** oo (+20-30%), map_set (+10-15%)
**Effort:** Medium

Property access currently does a linear scan of `PropertyStorage` on every get/set. An inline cache (IC) stores the last successfully looked-up property index and checks it first before scanning.

- Add a `last_property_index: u16` field to `HeapValue::Object`
- On GetProperty/SetProperty, check if the key matches the cached index
- If hit, skip the linear scan entirely
- If miss, do the full scan and update the cache

For objects with stable shapes (like the oo benchmark's constructor pattern), this should give a significant speedup.

### Phase 8.3: Fix closures TIMEOUT
**Target:** closures (∞→<50ms)
**Effort:** High (debugging)

The closures benchmark creates 1M closures and calls each once. This hangs completely. Need to investigate:
- Is there an unbounded allocation in closure creation?
- Is the `MakeClosure` instruction doing excessive work?
- Is there a memory leak in the closure environment?
- Profile with `cargo-flamegraph` to find the hot spot

### Phase 8.4: Optimize Map internal storage
**Target:** map_set (+30-50%)
**Effort:** High

The current Map uses `FxHashMap<String, Value>`. For 50K entries, this involves:
- 50K heap allocations for String keys
- Hash computation on every set/get
- Load factor management and rehashing

Options:
- **String interning:** Store strings once in an intern table, use indices in the map
- **FlatMap for small maps:** Use a sorted Vec<(K,V)> for maps with <32 entries
- **Pre-allocated capacity:** Reserve capacity upfront when Map size is known

### Phase 8.5: Generator suspend/resume optimization
**Target:** generators (+30-50%)
**Effort:** Medium

Each `yield` suspends the generator frame and each `next()` resumes it. Currently this involves:
- Saving/restoring the entire call frame
- Allocating a new heap value for the yield result
- Microtask queue interaction

Options:
- **Stackful generators:** Keep the generator frame on the native stack (like Lua coroutines)
- **Packed yield values:** Avoid heap allocation for simple yield values
- **Inline yield:** For generators with a single yield point, skip frame save/restore

### Phase 8.6: Promise microtask batching
**Target:** promises (+20-30%), promise_chain (+30-40%)
**Effort:** Medium

Currently each promise resolution drains the microtask queue. For promise chains, this means:
- Chain link 1 resolves → drain queue → chain link 2 resolves → drain queue → ...

Batch all synchronous resolutions before draining:
- When resolving a promise that has `.then()` callbacks, mark the callbacks as pending
- Don't drain the microtask queue until the current synchronous execution completes
- Process all pending callbacks in a single batch

### Phase 8.7: RegExp compilation cache
**Target:** regexp (+20-40%)
**Effort:** Low-Medium

The regexp benchmark creates the same regex pattern 1000 times in a loop. If the regex engine re-parses the pattern each time, caching the compiled pattern would help.

- Add a pattern→compiled cache in the RegExp heap value
- On `RegExp.exec`, check if the pattern hasn't changed and reuse the compiled form
- For static patterns (created once), this eliminates recompilation entirely

### Phase 8.8: Loop optimizations (continued)
**Target:** loops (+20-40%)
**Effort:** High

The fused LoopBranch helps but the interpreter still does ~320ns per iteration. Options:

- **Global-to-local promotion:** Detect `sum = sum + i` patterns in loops where `sum` is a global. Promote to a local variable and use `AddLocal` bytecode. Avoids the `LoadGlobal`/`StoreGlobal` overhead on every iteration.
- **Integer-specialized dispatch:** For loops with integer counters and limits, emit a specialized bytecode that avoids the `match` on Value types.
- **Computed goto:** Use GCC/Clang's `&&label` extension for threaded dispatch instead of a match statement.

### Phase 8.9: Reduce Value cloning in hot paths
**Target:** array_push (+10-20%), oo (+10-15%)
**Effort:** Medium

Many hot paths clone `Value` unnecessarily:
- `Array.push()` clones the value to store it
- `AddLocal` clones the result
- Property assignment clones the value

Use `Cow<Value>` or reference counting for values that are only read, not modified. For integer/float values, inline them in the enum to avoid heap allocation.

### Phase 8.10: JIT compilation (long-term)
**Target:** loops (+10-50x), closures (+10-50x)
**Effort:** Very High

The baseline JIT compiler infrastructure exists but is disabled. Once the profiler is proven correct:
- Enable JIT for hot loops (threshold=1000 iterations)
- Start with simple loops (integer counters, no side effects)
- Gradually expand to more complex patterns

## Priority Order

1. **Phase 8.3** (closures TIMEOUT) — blocking a benchmark from completing
2. **Phase 8.2** (inline caches) — broad impact on OO workloads
3. **Phase 8.6** (promise batching) — significant improvement for promise-heavy code
4. **Phase 8.4** (Map optimization) — addresses the 73x gap in map_set
5. **Phase 8.7** (RegExp cache) — low-hanging fruit for regexp workloads
6. **Phase 8.5** (generator optimization) — addresses the 56x gap
7. **Phase 8.8** (loop optimizations) — hardest problem, biggest absolute gap
8. **Phase 8.9** (reduce cloning) — incremental improvement across the board
9. **Phase 8.10** (JIT) — long-term, requires careful integration

## Wins to Protect

These optimizations are working well and should be preserved:
- DeferredResolve/DeferredReject for promises
- Microtask idle check
- ConsString for string concatenation
- has_accessors flag for property access
- Fused LoopBranch instruction
- SetProperty inline fast path (with accessor check)
- Lazy prototype allocation
