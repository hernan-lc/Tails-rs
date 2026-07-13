import { TestFn, HookFn, TestSuite } from './types.ts';
import { currentSuite, suiteStack, rootSuites } from './state.ts';

function getCurrentSuite(): TestSuite | null {
    if (suiteStack.length > 0) return suiteStack[suiteStack.length - 1];
    return currentSuite;
}

export function describe(name: string, fn: () => void): void {
    const suite: TestSuite = {
        name, fn, tests: [], beforeEach: [], afterEach: [],
        beforeAll: [], afterAll: [], children: [], skip: false,
    };
    const parent = getCurrentSuite();
    if (parent) {
        parent.children.push(suite);
    } else {
        rootSuites.push(suite);
    }
    suiteStack.push(suite);
    fn();
    suiteStack.pop();
}

export function it(name: string, fn: TestFn): void {
    const suite = getCurrentSuite();
    if (suite) suite.tests.push({ name, fn, skip: false });
}

export function test(name: string, fn: TestFn): void { it(name, fn); }

export function xit(name: string): void {
    const suite = getCurrentSuite();
    if (suite) suite.tests.push({ name, fn: () => {}, skip: true });
}

export function beforeAll(fn: HookFn): void {
    const suite = getCurrentSuite();
    if (suite) suite.beforeAll.push(fn);
}

export function afterAll(fn: HookFn): void {
    const suite = getCurrentSuite();
    if (suite) suite.afterAll.push(fn);
}

export function beforeEach(fn: HookFn): void {
    const suite = getCurrentSuite();
    if (suite) suite.beforeEach.push(fn);
}

export function afterEach(fn: HookFn): void {
    const suite = getCurrentSuite();
    if (suite) suite.afterEach.push(fn);
}
