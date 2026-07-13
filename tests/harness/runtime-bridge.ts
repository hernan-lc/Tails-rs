// Runtime bridge for Tails test runner
// This file is compiled into the test_runner binary

import { describe, it, test, xit, beforeAll, afterAll, beforeEach, afterEach, assert, runTests } from "./simple-harness.ts";

// Set up globals for test files
globalThis.describe = describe;
globalThis.it = it;
globalThis.test = test;
globalThis.xit = xit;
globalThis.beforeAll = beforeAll;
globalThis.afterAll = afterAll;
globalThis.beforeEach = beforeEach;
globalThis.afterEach = afterEach;
globalThis.assert = assert;
