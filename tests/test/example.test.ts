// Test file uses globals injected by the test runner
// Globals available: describe, it, test, xit, assert, beforeAll, afterAll, beforeEach, afterEach

describe("Math Operations", () => {
    it("should add two numbers", () => {
        assert.equal(2 + 2, 4);
    });

    it("should subtract two numbers", () => {
        assert.equal(5 - 3, 2);
    });

    it("should multiply two numbers", () => {
        assert.equal(3 * 4, 12);
    });

    it("should divide two numbers", () => {
        assert.equal(10 / 2, 5);
    });
});

describe("String Operations", () => {
    it("should concatenate strings", () => {
        assert.equal("Hello" + " " + "World", "Hello World");
    });

    it("should get string length", () => {
        assert.equal("Hello".length, 5);
    });
});

describe("Truthy/Falsy", () => {
    it("should be truthy", () => {
        assert.ok(true);
        assert.ok(1);
        assert.ok("hello");
    });

    it("should be falsy", () => {
        assert.notOk(false);
        assert.notOk(0);
        assert.notOk("");
    });
});
