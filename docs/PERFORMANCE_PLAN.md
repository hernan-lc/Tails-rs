# Tails-rs Performance Optimization Plan

## Benchmark Results Summary

### Cross-Runtime Comparison (Tails vs Node.js vs Bun)

| Benchmark | Tails | Node | Bun | Tails/Node | Tails/Bun |
|-----------|-------|------|-----|-----------|-----------|
| closures (1M) | 6434ms | 13ms | 27ms | **495x** | 238x |
| map_set (50K) | 780ms | 14ms | 15ms | **56x** | 52x |
| loops (5M) | 1223ms | 33ms | 8ms | **37x** | 153x |
| promises (100K) | 1070ms | 27ms | 29ms | **40x** | 37x |
| generators (2K) | 404ms | 10ms | 11ms | **40x** | 37x |
| promise_chain (500) | 87ms | 5ms | 3ms | **17x** | 29x |
| regexp (1000) | 1345ms | 80ms | 102ms | **17x** | 13x |
| oo (100K) | 1104ms | 130ms | 29ms | **8x** | 38x |
| array_push (200K) | 59ms | 9ms | 5ms | **7x** | 12x |
| date (500K) | 427ms | 170ms | 30ms | **2.5x** | 14x |
| json_parse (20) | 340ms | 122ms | 105ms | **3x** | 3x |
| async_await (50K) | 20ms | 9ms | 6ms | **2x** | 3x |
| string_concat (50K) | **TIMEOUT** | 6ms | 2ms | **∞** | ∞ |

### Rust Criterion Benchmarks (Internal)

| Benchmark | Time | Notes |
|-----------|------|-------|
| eval_hello_world | 1.9µs | Parse+eval overhead |
| eval_arithmetic_100 | 44µs | Loop + integer add |
| eval_arithmetic_1000 | 265µs | Scales linearly |
| eval_object_creation_20 | 28µs | Object + property set |
| eval_array_push_20 | 19µs | Array push |
| eval_array_push_100 | 59µs | Scales linearly |
| eval_fib_10 | 84µs | Recursive calls |
| eval_call_sum_100 | 78µs | Function calls in loop |
| eval_string_concat_20 | 81µs | String concat (ConsString) |
| eval_string_concat_50 | 418µs | **Non-linear scaling** |
| eval_string_concat_local_20 | 28µs | Local vars faster |
| eval_loop_only_1000 | 171µs | Pure loop dispatch |
| eval_nested_loop_50x50 | 687µs | Nested loop overhead |
| eval_json_parse | 20µs | JSON.parse |

## Root Cause Analysis

### Tier 1: Critical Bottlenecks (biggest impact)

#### 1. Closure Creation and Invocation (495x slower)

**Root cause:** Closures use `Rc<RefCell<Vec<Value>>>` for captured variables.
- Every closure creation allocates `Rc::new(RefCell::new(Vec::new()))` on the heap
- Every function call clones all captured variables: `closure_vars.borrow().iter().cloned()`
- `RefCell::borrow()` adds runtime borrow-checking on every call
- `CallFrame` carries a `HashMap<u32, Rc<RefCell<Vec<Value>>>>` per frame

**Impact:** `closures.js` (1M iterations of create+call) takes 6.4s vs 13ms on Node.

#### 2. String Concatenation Hangs (50K iterations)

**Root cause:** `ConsString` rope has exponential flatten cost.
- Each concat creates a new `ConsString` node with `Box<Value> + Box<Value>`
- No memoization of flattened result
- Every operation (equality, hash, comparison, number coercion, property keys) calls `flatten()` which allocates a new `String`
- For `"a" + "b" + "c" + ...` in a loop, the tree becomes deeply nested
- Flattening a tree of depth N requires O(N) allocation and traversal
- The benchmark calls `s = s + 'x'` 50K times, creating a tree of depth 50K
- When the loop ends, `console.log(s.length)` forces a full flatten of the 50K-deep tree
- Additional flatten calls in `to_string_coerce()` compound the problem

**Impact:** Hangs indefinitely on 50K iterations.

#### 3. Property Access (Map/Set 56x slower, OO 8x slower)

**Root cause:** No inline caches or hidden classes.
- Every `GetProperty` does a full `FxHashMap` lookup with string keys
- `key_to_str()` allocates a `String` for every property access
- ConsString keys force full flatten before hash lookup
- Map/Set methods go through: `GetProperty` → string match → `NativeFunction` → `call_value()` (3+ layers of indirection)
- `find_accessor` scans ALL properties on every miss (even when no accessors exist)

**Impact:** `map_set.js` (50K set+iterate) takes 780ms vs 14ms on Node.

#### 4. Loop Dispatch Overhead (37x slower)

**Root cause:** Each instruction dispatch goes through a large `match` with cold paths.
- Hot-path instructions are inlined but the cold path chains through `exec_load_store` → `exec_arithmetic` → `exec_comparison` → etc.
- Each non-matched branch adds function call overhead
- GC trigger check (`pc & 127 == 0`) happens every iteration

**Impact:** `loops.js` (5M iterations) takes 1.2s vs 33ms on Node.

### Tier 2: Significant Bottlenecks

#### 5. Value Type Size (32+ bytes)

**Root cause:** `Value` enum has 25 variants, largest is `String(String)` at 24 bytes.
- Every `clone()` is a 32+ byte memcpy
- Happens thousands of times per second on stack push/pop
- `Hash` for `ConsString` allocates and flattens on every map lookup
- `PartialEq` for `Cons+Cons` allocates two temporary strings

#### 6. Promise/Async Overhead (37-40x slower)

**Root cause:** Promise creation and resolution involves heavy allocation.
- Each `Promise.resolve()` allocates a new heap object
- `.then()` chaining creates intermediate promise objects
- Async/await creates generator-like state machines with heap-allocated state

#### 7. Generator Overhead (40x slower)

**Root cause:** Generators allocate state on the heap.
- Each `gen()` call creates a new generator object
- `yield` suspends the call frame to heap storage
- Resume restores from heap back to stack

## Optimization Roadmap

### Phase 1: String Concatenation Fix (HIGHEST PRIORITY)

**Goal:** Eliminate the string concat hang, target <10x of Bun for string operations.

| Task | Description | Expected Impact |
|------|-------------|-----------------|
| 1.1 | Memoize `ConsString::flatten()` — store `Option<String>` cache, compute once | Eliminates repeated O(N) flatten |
| 1.2 | Short-string eager flatten — if both operands are short (<64 bytes), flatten immediately instead of building a tree | Reduces tree depth for small concat |
| 1.3 | Avoid flatten in `PartialEq`/`Hash` — compare lengths first, then flatten only if lengths match; cache hash in ConsString | Reduces unnecessary allocs |
| 1.4 | Add flatten-on-demand for property key lookups — use a temporary buffer instead of allocating a new String | Eliminates per-access alloc |

**Verification:** `string_concat.js` (50K) should complete in <500ms.

### Phase 2: Closure and Function Call Optimization

**Goal:** Get closures within 5x of Node.js.

| Task | Description | Expected Impact |
|------|-------------|-----------------|
| 2.1 | Replace `Rc<RefCell<Vec<Value>>>` with stack-index-based capture — emit `LoadCaptured(slot)` instructions that read from the enclosing frame's stack directly | Eliminates heap alloc per closure |
| 2.2 | Avoid cloning captured vars on call — if closure env is stack-based, no cloning needed | Eliminates O(N) clone per call |
| 2.3 | Slim down `CallFrame` — intern `source_name` as `u32`, remove `shared_closure_env` from per-frame data | Reduces frame size, faster push/pop |
| 2.4 | Inline args in Call fast path — args are already on the stack, use move semantics instead of clone | Eliminates arg cloning |

**Verification:** `closures.js` (1M) should complete in <200ms.

### Phase 3: Value Type Shrinking

**Goal:** Reduce `Value` from 32+ bytes to 16 bytes.

| Task | Description | Expected Impact |
|------|-------------|-----------------|
| 3.1 | Intern all strings — replace `String(String)` with `String(u32)` pointing to a global intern table | Halves Value size |
| 3.2 | Move ConsString to the intern table — flatten and intern on creation | Consistent string handling |
| 3.3 | Use `#[repr(C)]` union layout for Value — 8-byte payload + 1-byte tag | Predictable memory layout |
| 3.4 | Update all clone sites to use `Copy` where possible | Eliminates memcpy on hot paths |

**Verification:** All criterion benchmarks should show 20-40% improvement.

### Phase 4: Property Access Optimization

**Goal:** Get property access within 5x of Node.js.

| Task | Description | Expected Impact |
|------|-------------|-----------------|
| 4.1 | Intern common property keys — "length", "name", "prototype", "constructor", "value", "done", "next", "push", "pop" as u32 IDs | Eliminates string alloc per access |
| 4.2 | Inline cache for GetProperty — store last successful hidden-class + offset at each callsite | Reduces hash lookup to type check + offset |
| 4.3 | Hidden classes / Shapes — objects with same key layout share a shape descriptor with fixed offsets | Eliminates hash lookup entirely for monomorphic access |
| 4.4 | Dedicated Map/Set bytecodes — `MapGet`, `MapSet`, `MapHas`, `SetAdd` instructions | Eliminates 3 layers of indirection |
| 4.5 | Fix WeakMap/WeakSet — replace linear `Vec` scan with `FxHashMap`/`FxHashSet` | O(N) → O(1) |
| 4.6 | Avoid `Vec` allocation in `PropertyStorage::iter()` — return true iterators | Eliminates alloc on iteration |

**Verification:** `map_set.js` and `oo.js` should improve by 10-20x.

### Phase 5: Loop and Dispatch Optimization

**Goal:** Get loop performance within 10x of Node.js.

| Task | Description | Expected Impact |
|------|-------------|-----------------|
| 5.1 | Computed dispatch table — replace match chain with lookup table indexed by instruction discriminant | Reduces dispatch to single indirect jump |
| 5.2 | Bytecode-level optimizations — fuse `LoadLocal + Add + StoreLocal` into `AddLocal` (partially done, extend) | Eliminates redundant stack ops |
| 5.3 | JIT-compile hot loops — detect loop back-edges, compile to native code for inner loops | 10-100x for tight loops |
| 5.4 | Reduce GC pressure — use bump allocator for young objects, mark-and-sweep without stack cloning | Reduces GC pauses |

**Verification:** `loops.js` (5M) should complete in <200ms.

### Phase 6: Promise/Async/Generator Optimization

**Goal:** Get async operations within 5x of Node.js.

| Task | Description | Expected Impact |
|------|-------------|-----------------|
| 6.1 | Optimize Promise.resolve() for already-resolved values — skip allocation, resolve synchronously | Reduces alloc for common case |
| 6.2 | Batch microtask execution — collect resolved promises, execute .then() callbacks in batch | Reduces per-promise overhead |
| 6.3 | Optimize generator suspend/resume — use stack-based save/restore instead of heap allocation | Reduces generator overhead |
| 6.4 | Async/await as syntactic sugar over generators — share implementation, optimize both | Reduces code duplication |

**Verification:** `promises.js`, `async_await.js`, `generators.js` should improve by 5-10x.

### Phase 7: Build System and Infrastructure

| Task | Description | Expected Impact |
|------|-------------|-----------------|
| 7.1 | Fix benchmark runner to handle Tails output format — parse `[tails] Script finished in Xms.` | Enables automated cross-runtime comparison |
| 7.2 | Add `--bench` mode to Tails binary — output raw timing data in machine-readable format | Enables CI benchmark tracking |
| 7.3 | Add criterion baseline tracking — store baseline numbers, detect regressions | Prevents performance regressions |
| 7.4 | Profile-guided optimization (PGO) — build with PGO using benchmark workloads | 10-20% across the board |

## Priority Order

1. **Phase 1** (String concat) — Fixes a hang, highest user-facing impact
2. **Phase 2** (Closures) — 495x gap, biggest absolute slowdown
3. **Phase 3** (Value type) — Foundation for all other optimizations
4. **Phase 4** (Property access) — Affects Map/Set, OO, and general property access
5. **Phase 5** (Loop dispatch) — Affects all loop-heavy code
6. **Phase 6** (Async) — Affects Promise-heavy code
7. **Phase 7** (Infrastructure) — Enables ongoing performance tracking

## Measuring Success

After each phase, re-run both benchmark suites:

```bash
# Cross-runtime
WARMUP=2 RUNS=5 TIMEOUT=60 bash benchmarks/runner.sh

# Rust criterion
cargo bench --bench runtime
```

Target milestones:
- **After Phase 1:** string_concat.js completes (no hang)
- **After Phase 2:** closures.js within 20x of Node
- **After Phase 3:** All benchmarks improve 20-40%
- **After Phase 4:** map_set.js within 10x of Node
- **After Phase 5:** loops.js within 10x of Node
- **After Phase 6:** async/await within 5x of Node
- **Overall goal:** Average Tails/Node ratio <10x across all benchmarks
