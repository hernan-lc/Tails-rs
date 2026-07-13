import { formatValue } from './utils.ts';

export const assert = {
    equal(actual: any, expected: any, message?: string): void {
        if (actual !== expected) {
            throw new Error(message || `Expected ${formatValue(expected)}, but got ${formatValue(actual)}`);
        }
    },

    strictEqual(actual: any, expected: any, message?: string): void {
        if (actual !== expected) {
            throw new Error(message || `Expected ${formatValue(expected)}, but got ${formatValue(actual)}`);
        }
    },

    deepEqual(actual: any, expected: any, message?: string): void {
        const a = JSON.stringify(actual);
        const e = JSON.stringify(expected);
        if (a !== e) throw new Error(message || `Expected ${e}, but got ${a}`);
    },

    notEqual(actual: any, expected: any, message?: string): void {
        if (actual === expected) throw new Error(message || `Expected values to be different`);
    },

    ok(value: any, message?: string): void {
        if (!value) throw new Error(message || `Expected truthy value, but got ${formatValue(value)}`);
    },

    notOk(value: any, message?: string): void {
        if (value) throw new Error(message || `Expected falsy value, but got ${formatValue(value)}`);
    },

    isType(value: any, type: string, message?: string): void {
        if (typeof value !== type) throw new Error(message || `Expected type "${type}", but got "${typeof value}"`);
    },

    isNull(value: any, message?: string): void {
        if (value !== null) throw new Error(message || `Expected null`);
    },

    isNotNull(value: any, message?: string): void {
        if (value === null) throw new Error(message || `Expected value to not be null`);
    },

    throws(fn: () => void, message?: string): void {
        let threw = false;
        try { fn(); } catch (e) { threw = true; }
        if (!threw) throw new Error(message || 'Expected function to throw');
    },

    doesNotThrow(fn: () => void, message?: string): void {
        try { fn(); } catch (e) {
            const s = e instanceof Error ? e.message : String(e);
            throw new Error(message || `Expected function not to throw, but threw: ${s}`);
        }
    },

    includes(array: any[], value: any, message?: string): void {
        if (!array.includes(value)) throw new Error(message || `Expected array to include ${formatValue(value)}`);
    },

    greaterThan(actual: number, expected: number, message?: string): void {
        if (actual <= expected) throw new Error(message || `Expected ${actual} > ${expected}`);
    },

    lessThan(actual: number, expected: number, message?: string): void {
        if (actual >= expected) throw new Error(message || `Expected ${actual} < ${expected}`);
    },

    fail(message: string): void { throw new Error(`Assertion failed: ${message}`); },
};
