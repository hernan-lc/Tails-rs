// Embedded Test Harness for Tails-rs
//
// NOTE: This harness avoids `for...of` and `await` in loops because the
// Tails-rs VM has two known issues:
//   1. `for...of` + `await` corrupts the iterator state (reading '"next"')
//   2. When `try/catch` catches a throw, local variables on the VM stack
//      are lost (become undefined). Module-level variables are preserved.
//
// To work around #2, all test-running state that must survive a catch
// boundary is stored at module (global) scope, not in locals.
type TestFn = () => void;
type HookFn = () => void;

interface TestCase {
    name: string;
    fn: TestFn;
    skip: boolean;
}

interface TestSuite {
    name: string;
    fn: () => void;
    tests: TestCase[];
    beforeEach: HookFn[];
    afterEach: HookFn[];
    beforeAll: HookFn[];
    afterAll: HookFn[];
    children: TestSuite[];
    skip: boolean;
}

// Module-level state (survives try/catch boundaries)
let _currentSuite: TestSuite | null = null;
const _rootSuites: TestSuite[] = [];
const _suiteStack: TestSuite[] = [];
let _passed = 0;
let _failed = 0;
let _skipped = 0;
const _failures: string[] = [];
// Per-test state (set before try/catch so catch block can read it)
let _curTestName = '';
let _curFullName = '';
let _curStart = 0;
let _curErrorMsg = '';

function _getCurrentSuite(): TestSuite | null {
    if (_suiteStack.length > 0) return _suiteStack[_suiteStack.length - 1];
    return _currentSuite;
}

function _formatValue(val: any): string {
    if (val === null) return 'null';
    if (val === undefined) return 'undefined';
    if (typeof val === 'string') return JSON.stringify(val);
    if (typeof val === 'number' || typeof val === 'boolean') return String(val);
    if (typeof val === 'function') return '[Function]';
    if (val instanceof Error) return val.name + ': ' + val.message;
    try {
        if (Array.isArray(val)) return '[' + val.map(_formatValue).join(', ') + ']';
        if (typeof val === 'object') {
            const entries = Object.entries(val).map(([k, v]) => k + ': ' + _formatValue(v));
            return '{' + entries.join(', ') + '}';
        }
        return String(val);
    } catch {
        return String(val);
    }
}

function describe(name: string, fn: () => void): void {
    const suite: TestSuite = {
        name, fn, tests: [], beforeEach: [], afterEach: [],
        beforeAll: [], afterAll: [], children: [], skip: false,
    };
    const parent = _getCurrentSuite();
    if (parent) parent.children.push(suite);
    else _rootSuites.push(suite);
    _suiteStack.push(suite);
    fn();
    _suiteStack.pop();
}

function it(name: string, fn: TestFn): void {
    const suite = _getCurrentSuite();
    if (suite) suite.tests.push({ name, fn, skip: false });
}

function test(name: string, fn: TestFn): void { it(name, fn); }

function xit(name: string): void {
    const suite = _getCurrentSuite();
    if (suite) suite.tests.push({ name, fn: () => {}, skip: true });
}

function beforeAll(fn: HookFn): void {
    const suite = _getCurrentSuite();
    if (suite) suite.beforeAll.push(fn);
}

function afterAll(fn: HookFn): void {
    const suite = _getCurrentSuite();
    if (suite) suite.afterAll.push(fn);
}

function beforeEach(fn: HookFn): void {
    const suite = _getCurrentSuite();
    if (suite) suite.beforeEach.push(fn);
}

function afterEach(fn: HookFn): void {
    const suite = _getCurrentSuite();
    if (suite) suite.afterEach.push(fn);
}


const assert = {
    equal(actual: any, expected: any, message?: string): void {
        if (actual !== expected) throw new Error(message || 'Expected ' + _formatValue(expected) + ', but got ' + _formatValue(actual));
    },
    strictEqual(actual: any, expected: any, message?: string): void {
        if (actual !== expected) throw new Error(message || 'Expected ' + _formatValue(expected) + ', but got ' + _formatValue(actual));
    },
    deepEqual(actual: any, expected: any, message?: string): void {
        const a = JSON.stringify(actual);
        const e = JSON.stringify(expected);
        if (a !== e) throw new Error(message || 'Expected ' + e + ', but got ' + a);
    },
    notEqual(actual: any, expected: any, message?: string): void {
        if (actual === expected) throw new Error(message || 'Expected values to be different');
    },
    ok(value: any, message?: string): void {
        if (!value) throw new Error(message || 'Expected truthy value, but got ' + _formatValue(value));
    },
    notOk(value: any, message?: string): void {
        if (value) throw new Error(message || 'Expected falsy value, but got ' + _formatValue(value));
    },
    isType(value: any, type: string, message?: string): void {
        if (typeof value !== type) throw new Error(message || 'Expected type ' + type + ', but got ' + typeof value);
    },
    throws(fn: () => void, message?: string): void {
        let threw = false;
        try { fn(); } catch (e) { threw = true; }
        if (!threw) throw new Error(message || 'Expected function to throw');
    },
    doesNotThrow(fn: () => void, message?: string): void {
        try { fn(); } catch (e) {
            const s = e instanceof Error ? e.message : String(e);
            throw new Error(message || 'Expected function not to throw, but threw: ' + s);
        }
    },
    includes(array: any[], value: any, message?: string): void {
        if (!array.includes(value)) throw new Error(message || 'Expected array to include ' + _formatValue(value));
    },
    greaterThan(actual: number, expected: number, message?: string): void {
        if (actual <= expected) throw new Error(message || 'Expected ' + actual + ' > ' + expected);
    },
    lessThan(actual: number, expected: number, message?: string): void {
        if (actual >= expected) throw new Error(message || 'Expected ' + actual + ' < ' + expected);
    },
    fail(message: string): void { throw new Error('Assertion failed: ' + message); },
};


async function _runSuite(suite: TestSuite, prefix: string): Promise<void> {
    const fullName = prefix ? prefix + ' > ' + suite.name : suite.name;
    if (suite.name) console.log('\n📦 ' + fullName);

    // Synchronous test execution using module-level state to avoid
    // local-variable corruption after try/catch (Tails-rs VM limitation).

    // beforeAll hooks
    for (let i = 0; i < suite.beforeAll.length; i++) {
        try {
            suite.beforeAll[i]();
        } catch (e) {
            const s = e instanceof Error ? e.message : String(e);
            _failures.push('[beforeAll in ' + fullName + '] ' + s);
            _failed += suite.tests.length;
            return;
        }
    }

    // Tests
    for (let i = 0; i < suite.tests.length; i++) {
        const test = suite.tests[i];
        if (test.skip) { console.log('  ⏭️  SKIPPED: ' + test.name); _skipped++; continue; }

        // beforeEach hooks
        let beforeEachFailed = false;
        let beforeEachError = '';
        for (let j = 0; j < suite.beforeEach.length; j++) {
            try {
                suite.beforeEach[j]();
            } catch (e) {
                beforeEachFailed = true;
                beforeEachError = e instanceof Error ? e.message : String(e);
                break;
            }
        }

        if (beforeEachFailed) {
            console.log('  ❌ ' + test.name + ' (beforeEach failed)');
            _failures.push('[' + fullName + '] ' + test.name + ': ' + beforeEachError);
            _failed++;
            continue;
        }

        // Run test — store state at module level before try/catch
        _curTestName = test.name;
        _curFullName = fullName;
        _curStart = Date.now();
        _curErrorMsg = '';
        try {
            test.fn();
            // Success — locals may be lost after try, read from module-level _cur*
            const elapsed = Date.now() - _curStart;
            console.log('  ✅ ' + _curTestName + ' (' + elapsed + 'ms)');
            _passed++;
        } catch (e) {
            _curErrorMsg = e instanceof Error ? e.message : String(e);
            const elapsed = Date.now() - _curStart;
            console.log('  ❌ ' + _curTestName + ' (' + elapsed + 'ms)');
            console.log('     ' + _curErrorMsg);
            _failures.push('[' + _curFullName + '] ' + _curTestName + ': ' + _curErrorMsg);
            _failed++;
        }

        // afterEach hooks
        for (let j = 0; j < suite.afterEach.length; j++) {
            try {
                suite.afterEach[j]();
            } catch (e) {
                const s = e instanceof Error ? e.message : String(e);
                console.log('     ⚠️  afterEach failed: ' + s);
            }
        }
    }

    // Child suites
    for (let i = 0; i < suite.children.length; i++) {
        _runSuite(suite.children[i], fullName);
    }

    // afterAll hooks
    for (let i = 0; i < suite.afterAll.length; i++) {
        try {
            suite.afterAll[i]();
        } catch (e) {
            const s = e instanceof Error ? e.message : String(e);
            console.log('  ⚠️  [afterAll] ' + s);
        }
    }
}

function runTests(): void {
    _passed = 0; _failed = 0; _skipped = 0;
    _failures.length = 0;
    console.log('\n🧪 Running tests...\n');
    for (let i = 0; i < _rootSuites.length; i++) {
        _runSuite(_rootSuites[i], '');
    }
    console.log('\n' + '='.repeat(50));
    console.log('Results: ' + _passed + ' passed, ' + _failed + ' failed, ' + _skipped + ' skipped');
    console.log('='.repeat(50));
    if (_failures.length > 0) {
        console.log('\nFailures:');
        _failures.forEach((f, i) => console.log('  ' + (i + 1) + '. ' + f));
    }
    globalThis.__TEST_RESULTS__ = { passed: _passed, failed: _failed, skipped: _skipped };
}
