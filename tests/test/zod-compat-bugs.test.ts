// Zod Runtime Compatibility Tests — KNOWN BUGS
//
// These tests expose specific VM bugs that prevent full Zod compatibility.
// They are expected to FAIL. Each test documents one bug.
//
// When a bug is fixed, move the corresponding test to zod-compat.test.ts.

describe("BUG: static method constructor reference", () => {
    // Error: "42 is not a constructor"
    // The VM resolves the class name inside a static method to
    // the function argument instead of the class binding.
    it("static method can construct own class", () => {
        class Factory {
            static create(value: number) { return new Factory(value); }
            value: number;
            constructor(value: number) { this.value = value; }
        }
        const instance = Factory.create(42);
        assert.equal(instance.value, 42);
    });
});

describe("BUG: class chain static factory", () => {
    // Error: "X is not a constructor"
    // Inherited static factories fail because the parent class
    // name doesn't resolve correctly in the subclass context.
    it("subclass static factory creates instance", () => {
        class ZodType {
            static create() { return new ZodType(); }
            _type: string;
            constructor() { this._type = 'base'; }
        }
        class ZodString extends ZodType {
            constructor() { super(); }
            static create() { return new ZodString(); }
        }
        const instance = ZodString.create();
        assert.equal(instance._type, 'base');
        assert.ok(instance instanceof ZodString);
        assert.ok(instance instanceof ZodType);
    });
});

describe("BUG: non-enumerable descriptor not preserved", () => {
    // Error: Expected false, but got true
    // getOwnPropertyDescriptors returns enumerable:true even when
    // the property was defined with enumerable:false.
    it("descriptor preserves enumerable flag", () => {
        const obj: any = {};
        Object.defineProperty(obj, 'hidden', {
            value: 42,
            enumerable: false
        });
        const desc = Object.getOwnPropertyDescriptors(obj);
        assert.ok(desc.hidden !== undefined);
        assert.equal(desc.hidden.value, 42);
        assert.equal(desc.hidden.enumerable, false);
    });
});

describe("BUG: spread with custom Symbol.iterator", () => {
    // Error: Expected 3, but got 0
    // The spread operator [...iterable] produces an empty array
    // when the iterable uses a custom Symbol.iterator.
    it("spread collects custom iterator values", () => {
        const iterable: any = {
            _values: [1, 2, 3],
            [Symbol.iterator]() {
                let i = 0;
                const values = this._values;
                return {
                    next() {
                        return i < values.length
                            ? { value: values[i++], done: false }
                            : { value: undefined, done: true };
                    }
                };
            }
        };
        const result = [...iterable];
        assert.equal(result.length, 3);
        assert.equal(result[0], 1);
        assert.equal(result[1], 2);
        assert.equal(result[2], 3);
    });
});

describe("BUG: Array.from with mapper produces empty array", () => {
    // Error: Expected 3, but got 0
    // Array.from({ length: N }, mapper) returns an empty array
    // instead of mapping over the length.
    it("Array.from creates array from iterable with mapper", () => {
        const arr = Array.from({ length: 3 }, (_, i) => i * 2);
        assert.equal(arr.length, 3);
        assert.equal(arr[0], 0);
        assert.equal(arr[1], 2);
        assert.equal(arr[2], 4);
    });
});

describe("BUG: Array.prototype.at returns wrong value", () => {
    // Error: Expected 1, but got false
    // arr.at(0) returns false instead of the element at index 0.
    it("at() returns element at positive index", () => {
        const arr = [1, 2, 3];
        assert.equal(arr.at(0), 1);
        assert.equal(arr.at(-1), 3);
    });
});

describe("BUG: for...of iterator corruption", () => {
    // Error: "Cannot read properties of undefined (reading 'next')"
    // for...of over Object.keys with discarded expressions and
    // method calls in the body causes iterator corruption.
    it("for...of with discarded expr and method call", () => {
        const doc = { write: (s: string) => s.length };
        const normalized = { keys: ['age', 'status', 'role', 'email', 'active', 'id', 'tags', 'name', 'nickname'] };
        doc;
        normalized;
        let total = 0;
        for (const key of normalized.keys) {
            total += doc.write(key);
        }
        assert.ok(total > 0);
    });
});
