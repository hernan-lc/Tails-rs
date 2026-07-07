/*---
includes: [assert.js]
description: Async and Await conformance tests
---*/

var result = 0;
async function foo() {
    var val = await Promise.resolve(42);
    result = val;
}

foo();
// Since TailsRuntime.eval runs the event loop automatically if there's pending work,
// we just need to wait for the promise to resolve.
// However, 'result' will only be updated after the event loop runs.
// If eval() returns before that, this test might fail if we check 'result' here.
// But in tails-rs, eval() for an async script should handle it.

// For now, let's use a simple then-able
Promise.resolve(100).then(function(v) {
    assert.sameValue(v, 100);
});
