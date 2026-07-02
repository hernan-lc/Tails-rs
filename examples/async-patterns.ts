// Async/await patterns example for Tails-rs
// Run: tails run examples/async-patterns.ts
//
// Demonstrates: async functions, await, and basic Promise usage.

// ---------------------------------------------------------------------------
// Basic async/await
// ---------------------------------------------------------------------------

async function fetchUser(id: number): Promise<string> {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve(`User-${id}`);
    }, 10);
  });
}

async function basicAsync() {
  console.log("=== Basic async/await ===");
  const user = await fetchUser(1);
  console.log("Got user:", user);
}

// ---------------------------------------------------------------------------
// Promise.resolve and Promise.reject
// ---------------------------------------------------------------------------

async function promiseBasics() {
  console.log("\n=== Promise basics ===");

  const resolved = await Promise.resolve("resolved value");
  console.log("Promise.resolve:", resolved);

  try {
    await Promise.reject("rejected value");
  } catch (e) {
    console.log("Promise.reject caught:", e);
  }
}

// ---------------------------------------------------------------------------
// Error handling with try/catch
// ---------------------------------------------------------------------------

async function errorHandling() {
  console.log("\n=== Error handling ===");
  try {
    await Promise.reject(new Error("something broke"));
  } catch (e: any) {
    console.log("Caught error:", e.message);
  }
}

// ---------------------------------------------------------------------------
// Sequential awaits
// ---------------------------------------------------------------------------

async function sequentialWaits() {
  console.log("\n=== Sequential awaits ===");
  const start = Date.now();
  const a = await fetchUser(1);
  const b = await fetchUser(2);
  const c = await fetchUser(3);
  const elapsed = Date.now() - start;
  console.log("Results:", a, b, c);
  console.log("Elapsed:", elapsed + "ms");
}

// ---------------------------------------------------------------------------
// Async function returning a value
// ---------------------------------------------------------------------------

async function computedValue(): Promise<number> {
  const a = await Promise.resolve(10);
  const b = await Promise.resolve(20);
  return a + b;
}

async function asyncReturnValue() {
  console.log("\n=== Async return value ===");
  const result = await computedValue();
  console.log("10 + 20 =", result);
}

// ---------------------------------------------------------------------------
// Run all examples
// ---------------------------------------------------------------------------

async function main() {
  await basicAsync();
  await promiseBasics();
  await errorHandling();
  await sequentialWaits();
  await asyncReturnValue();
  console.log("\n=== All async patterns completed ===");
}

main();
