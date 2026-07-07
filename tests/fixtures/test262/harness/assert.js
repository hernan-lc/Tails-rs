function assert(val, msg) {
    if (!val) {
        throw new Error("Assertion failed: " + (msg || ""));
    }
}

assert.sameValue = function(actual, expected, msg) {
    if (actual !== expected) {
        throw new Error("Expected " + expected + ", got " + actual + (msg ? ": " + msg : ""));
    }
};

assert.throws = function(expectedError, fn, msg) {
    try {
        fn();
    } catch (e) {
        // Simple check
        return;
    }
    throw new Error("Expected error to be thrown" + (msg ? ": " + msg : ""));
};
