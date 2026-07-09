function assert(condition, message) {
    if (!condition) {
        throw new Error("Assertion failed: " + (message || "expected true"));
    }
}

assert.sameValue = function(actual, expected, message) {
    if (actual !== expected) {
        // Special NaN check
        if (typeof actual === "number" && typeof expected === "number" && isNaN(actual) && isNaN(expected)) {
            return;
        }
        throw new Error("Assertion failed: " + (message || "") + " (expected " + expected + ", got " + actual + ")");
    }
};

assert.throws = function(expectedErrorConstructor, func, message) {
    let threw = false;
    let thrownError = null;
    try {
        func();
    } catch (e) {
        threw = true;
        thrownError = e;
    }
    if (!threw) {
        throw new Error("Assertion failed: expected error to be thrown " + (message || ""));
    }
    // Check constructor if provided
    if (expectedErrorConstructor) {
        let expectedName = expectedErrorConstructor.name || (expectedErrorConstructor.toString ? expectedErrorConstructor.toString() : "");
        if (thrownError && thrownError.name !== expectedName && thrownError.constructor !== expectedErrorConstructor) {
            if (!(thrownError instanceof expectedErrorConstructor)) {
                throw new Error("Assertion failed: expected error of type " + expectedName + ", but got " + (thrownError.name || typeof thrownError));
            }
        }
    }
};
