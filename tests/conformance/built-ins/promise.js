/*---
description: Tests Promises (resolve, reject, then, catch, finally, all, await, and setTimeout).
---*/

// Promise.resolve static
let resolvedVal = 0;
Promise.resolve(99).then(function(val) {
    resolvedVal = val;
});
assert.sameValue(resolvedVal, 99, "Promise.resolve static resolved value");

// Promise.reject static
let rejectedVal = "";
Promise.reject("err").catch(function(val) {
    rejectedVal = val;
});
assert.sameValue(rejectedVal, "err", "Promise.reject catch block handler");

// Promise.all resolved
let allLength = 0;
Promise.all([Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)]).then(function(val) {
    allLength = val.length;
});
assert.sameValue(allLength, 3, "Promise.all resolved array length");

// Promise.all one rejected
let allRejectedVal = "";
Promise.all([Promise.resolve(1), Promise.reject("fail"), Promise.resolve(3)]).catch(function(val) {
    allRejectedVal = val;
});
assert.sameValue(allRejectedVal, "fail", "Promise.all rejected handler");

// Promise finally
let finallyVal = 0;
Promise.resolve(1)
    .then(function(val) {
        finallyVal = finallyVal + val;
    })
    .finally(function() {
        finallyVal = finallyVal + 10;
    });
assert.sameValue(finallyVal, 11, "Promise finally execution");

// Promise chaining multiple thens
let chainVal = 0;
Promise.resolve(1)
    .then(function(val) {
        chainVal = chainVal + val;
    })
    .then(function() {
        chainVal = chainVal + 10;
    });
assert.sameValue(chainVal, 11, "Promise multiple thens chaining");
