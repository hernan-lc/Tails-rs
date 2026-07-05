# Phase 7: Closing the Loop and Promise Gaps

## Current State

| Benchmark | Tails (ms) | Bun (ms) | Gap | Priority |
|-----------|-----------|---------|-----|----------|
| loops | 1,802 | 8.3 | 217x | **P0** |
| promises | 2,014 | 27 | 74.6x | **P0** |
| promise_chain | 156 | 4 | 39x | P1 |
| oo | 1,461 | 26.7 | 54.7x | P1 |
| map_set | 926 | 16.7 | 55.5x | P1 |
| closures | TIMEOUT | 36 | ∞ | P1 |
| generators | 665 | 12 | 55.4x | P2 |
| regexp | 2,089 | 100 | 20.9x | P2 |
| string_concat | 39 | 2.7 | 14.4x | P2 |
| json_parse | 746 | 119 | 6.3x | P2 |
| date | 522 | 31.3 | 16.7x | P2 |
| array_push | 82 | 6.7 | 12.2x | P2 |
| async_await | 32 | 5.3 | 6x | P2 |

---

## Priority 1: Loops (217x → target <50x)

### Root Cause Analysis

The loops benchmark executes 12 instructions per iteration, 5M iterations = 60M instructions.
Estimated per-instruction cost: ~30ns (vs native ~0.3ns). The overhead breaks down as:

1. **Match dispatch** (~10ns/iter): Large `match` with 50+ variants, branch predictor thrashing
2. **Value enum cloning** (~8ns/iter): `Value::clone()` on every stack pop (Integer = memcpy + discriminant)
3. **`LoadGlobal`/`StoreGlobal` for `x`** (~30ns/iter): HashMap lookup for module-scope variable
4. **Stack Vec operations** (~5ns/iter): pop/push on Vec with capacity management
5. **No fused instructions**: IncLocal + compare + jump is 3 separate dispatches

### Task 7.1: Compute a dispatch table (jump table)

**What:** Replace the cascading `match` with a pre-computed `[fn; N]` dispatch table.
Instructions are indexed by opcode, so `dispatch[instruction.opcode()](self)` skips the
match entirely.

**How:**
```rust
type InstrFn = fn(&mut Interpreter, &Instruction, usize) -> Result<()>;
static DISPATCH: [InstrFn; 256] = [
    // filled at compile time with #[allow(unreachable)] fallbacks
    // for unused opcodes, and the actual handler fns for each opcode
];

// In the hot loop:
loop {
    if pc & 127 == 0 && self.gc.should_collect() {
        self.collect_garbage();
    }
    let instruction = &instructions[pc];
    DISPATCH[instruction.opcode()](self, instruction, pc)?;
    pc += 1;
}
```

**Impact:** ~1.3-1.5x (reduces dispatch from ~10ns to ~4ns by eliminating match overhead)

### Task 7.2: Eliminate Value::clone() on hot stack operations

**What:** Change `stack_pop()` to return `Value` by move (not clone), and change
`stack.last()` to return `&Value`. The only place that needs clone is
`self.stack[idx].clone()` for LoadLocal.

**Already done partially** (stack_pop already does `.pop()` = move). The remaining issue is:
- `self.stack.last().cloned()` in various places → change to `.last()` where possible
- `Add`/`Sub`/`Eq` inlined instructions pop by move (already correct)
- The `LoadLocal` instruction in the main match does `.clone()` on the stack value

**Concrete change:** In the inlined `LoadLocal` path, if the instruction is the last use,
avoid the clone by using `get_unchecked` + copy for Integer/Float (which are Copy types).

```rust
// Before:
Instruction::LoadLocal(slot) => {
    let val = self.stack[*slot as usize].clone();  // alloc for String
    self.stack.push(val);
}
// After:
Instruction::LoadLocal(slot) => {
    self.stack.push(self.stack[*slot as usize].clone());
    // Integer and Float are cheap to clone (memcpy). String is expensive
    // but not used in hot loops. Accept the cost for correctness.
}
```

Actually the clone is already correct and cheap for integers. The real issue is elsewhere.

### Task 7.3: Promote module-scope variables to locals (HIGHEST IMPACT for loops)

**What:** In the loops benchmark, `x` is declared at module scope. The compiler emits
`LoadGlobal("x")` and `StoreGlobal("x")` which do HashMap lookups (~15ns each). If we
detect that a global is only written by the current function, we can promote it to a
local variable.

**How (compiler-side):**
- Add a pass that analyzes module-level `let` bindings
- If a binding is only modified within a single function scope, emit it as a local
  variable in that function's frame instead of a global
- Fall back to global access for the general case

**How (VM-side, simpler):** Add a new instruction `LoadGlobalInt` / `StoreGlobalInt`
that uses an integer-indexed array instead of a HashMap:

```rust
// New instruction: store integer directly into indexed global slot
Instruction::StoreGlobalInt(slot_idx: u16, value: Value) => {
    self.global_slots[slot_idx as usize] = value;
}
```

The compiler assigns each module-scope `let` a slot index at compile time.

**Impact:** ~2-3x for loops (eliminates 2 HashMap lookups per iteration = ~30ns saved)

### Task 7.4: Add `AddGlobal(slot, LoadLocal(slot))` fused instruction

**What:** For the pattern `x = x + i` where `x` is a global and `i` is a local,
create a single fused instruction that:
1. Reads the global by slot index (no HashMap)
2. Reads the local by index
3. Adds them (Integer fast path)
4. Stores the result back to the global slot

```rust
Instruction::AddGlobalSlot(global_slot: u16, local_slot: u8) => {
    let left = self.global_slots[global_slot as usize].clone();
    let right = self.stack[local_slot as usize].clone();
    let result = self.add(left, right)?;
    self.global_slots[global_slot as usize] = result;
}
```

**Impact:** ~1.5x (replaces 4 instructions with 1)

### Task 7.5: Fuse loop control flow

**What:** For `for (let i = 0; i < N; i++)` patterns, emit a single
`LoopBranch(local_slot, const_idx, body_start)` instruction that:
1. Loads local
2. Loads constant
3. Compares (Less)
4. If true: jumps to body_start, increments local
5. If false: falls through

```rust
Instruction::LoopBranch {
    counter_slot: u8,
    limit_idx: u16,
    body_pc: u16,
    counter_delta: i32,  // increment amount (usually 1)
}
```

**Impact:** ~2-3x (replaces 5 instructions with 1)

### Loop Optimization Summary

| Task | Description | Estimated Speedup | Difficulty |
|------|-------------|------------------|------------|
| 7.1 | Dispatch table | 1.3-1.5x | Medium |
| 7.2 | Eliminate clone on hot path | 1.1x | Easy |
| 7.3 | Global-to-local promotion | 2-3x | Hard (compiler) |
| 7.4 | Fused AddGlobal instruction | 1.5x | Medium |
| 7.5 | Fused loop branch instruction | 2-3x | Medium |

**Combined estimate:** 217x → ~25-35x (with 7.3+7.5)
**Without 7.3 (VM-only):** 217x → ~50-70x (with 7.1+7.4+7.5)

---

## Priority 2: Promises (74.6x → target <20x)

### Root Cause Analysis

For `await new Promise(resolve => resolve(i))` × 100K:
- Each iteration: 4+ heap allocs (promise + resolve fn + reject fn + closure vec)
- Each `await`: full interpreter state snapshot (stack + call_stack + handlers)
- `drain_microtasks()` called every tick even when queue is empty

### Task 7.6: Skip await suspension for already-resolved promises (HIGHEST IMPACT)

**What:** When `Await` instruction encounters a promise that is already `Fulfilled`,
push the value directly without creating a `SuspendedFrame`.

```rust
Instruction::Await(promise_idx) => {
    let promise_val = self.stack.last().cloned().unwrap_or(Value::Undefined);
    if let Value::Promise(idx) = &promise_val {
        if let HeapValue::Promise(p) = &self.heap[*idx] {
            match &p.state {
                PromiseState::Fulfilled(value) => {
                    // Fast path: already resolved, skip suspension entirely
                    self.stack.pop(); // remove promise from stack
                    self.stack.push(value.clone());
                    pc += 1;
                    continue;
                }
                _ => { /* fall through to full suspend */ }
            }
        }
    }
}
```

**Impact:** 3-5x for the `promises.js` benchmark (100K instant-resolve pattern)

### Task 7.7: Lazy resolve/reject function creation

**What:** Instead of eagerly creating resolve/reject functions in `create_promise`,
only create them when the executor actually calls `resolve` or `reject`.

The current flow in the `Promise` constructor:
1. `create_resolve_fn(promise_idx)` → heap alloc #1
2. `create_reject_fn(promise_idx)` → heap alloc #2
3. Call executor(resolve, reject)
4. If resolve was called synchronously → state is already Fulfilled

The new flow:
1. Create a single "deferred" object with a reference to the promise
2. Pass a lightweight resolve/reject handle to the executor
3. Only create the full JsFunction if the executor stores the handle for later

**Concrete approach:** Add a `DeferredPromise` heap value that wraps the promise index.
When `resolve(value)` is called on it, it directly calls `resolve_promise` without
going through the full function call dispatch.

```rust
HeapValue::DeferredResolve(usize),  // promise_idx
HeapValue::DeferredReject(usize),   // promise_idx
```

When called as a function:
```rust
HeapValue::DeferredResolve(idx) => {
    resolve_promise(idx, args[0].clone());
    return Ok(Value::Undefined);
}
```

**Impact:** 1.5-2x (eliminates 2 JsFunction heap allocs per Promise creation)

### Task 7.8: Skip drain_microtasks when queue is idle

**What:** Add an `is_idle()` check to `AsyncRuntime` so `drain_microtasks()` returns
immediately when no microtasks are pending.

```rust
// In AsyncRuntime:
pub fn is_idle(&self) -> bool {
    self.microtask_queue.is_empty() && self.pending_promises.is_empty()
}

// In interpreter:
if !self.async_runtime.is_idle() {
    self.drain_microtasks();
}
```

**Impact:** 1.2-1.5x (avoids Vec allocation + iteration when queue is empty)

### Task 7.9: Avoid stack snapshot for simple await

**What:** Instead of `std::mem::take(&mut self.stack)` (which replaces the stack with
an empty Vec, requiring reallocation on resume), use a more efficient snapshot:

```rust
// Option A: Save only the PC, use a frame index to restore
SimplifiedFrame {
    promise_idx,
    resume_pc: pc + 1,
    stack_len: self.stack.len(),  // just save the length
    call_stack_len: self.call_stack.len(),
}

// On resume: truncate back to saved length instead of full clone
```

Actually this won't work because different awaits need different stack states.
The current approach is correct but expensive.

**Better approach:** Use a `VecDeque` of stack segments instead of full snapshot:
```rust
struct SuspendedFrame {
    promise_idx: usize,
    resume_pc: usize,
    stack_savepoint: usize,  // index into a shared stack arena
    call_stack_savepoint: usize,
}
```

This requires a more significant refactor but eliminates O(N) clone per await.

**Impact:** 1.5-2x for deep call stacks, minimal for shallow ones

### Task 7.10: Microtask batching

**What:** Instead of calling `drain_microtasks()` once per outer loop tick,
call it once per `execute_from` call (after the main bytecode loop returns):

```rust
// In execute():
loop {
    let result = self.execute_from(...);
    self.drain_microtasks();  // drain once after execution, not per-tick
    if !matches!(result, ...) {
        break;
    }
}
```

**Impact:** 1.3-1.5x (reduces drain_microtasks calls from ~100K to ~1)

### Promise Optimization Summary

| Task | Description | Estimated Speedup | Difficulty |
|------|-------------|------------------|------------|
| 7.6 | Skip await for resolved promises | 3-5x | Easy |
| 7.7 | Lazy resolve/reject fn creation | 1.5-2x | Medium |
| 7.8 | Skip drain when idle | 1.2-1.5x | Easy |
| 7.9 | Efficient stack snapshots | 1.5-2x | Hard |
| 7.10 | Microtask batching | 1.3-1.5x | Medium |

**Combined estimate:** 74.6x → ~10-15x

---

## Implementation Order

### Wave 1: Quick wins (both loops and promises)
1. **7.6** — Skip await for resolved promises (Easy, 3-5x for promises)
2. **7.8** — Skip drain_microtasks when idle (Easy, 1.2-1.5x for promises)
3. **7.1** — Dispatch table (Medium, 1.3-1.5x for loops)

### Wave 2: Medium effort, high impact
4. **7.7** — Lazy resolve/reject fn creation (Medium, 1.5-2x for promises)
5. **7.4** — Fused AddGlobal instruction (Medium, 1.5x for loops)
6. **7.5** — Fused loop branch instruction (Medium, 2-3x for loops)

### Wave 3: Hard but transformative
7. **7.3** — Global-to-local variable promotion (Hard, 2-3x for loops)
8. **7.9** — Efficient stack snapshots (Hard, 1.5-2x for promises)
9. **7.10** — Microtask batching (Medium, 1.3-1.5x for promises)

### Projected Results After All Waves

| Benchmark | Before | After Wave 1 | After Wave 2 | After Wave 3 |
|-----------|--------|-------------|-------------|-------------|
| loops | 217x | ~130x | ~40x | ~25x |
| promises | 74.6x | ~20x | ~14x | ~10x |
| promise_chain | 39x | ~25x | ~18x | ~12x |
