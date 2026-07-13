import { TestSuite } from './types.ts';

export let currentSuite: TestSuite | null = null;
export const rootSuites: TestSuite[] = [];
export const suiteStack: TestSuite[] = [];
export let passed = 0;
export let failed = 0;
export let skipped = 0;
export const failures: string[] = [];

export function setCurrentSuite(suite: TestSuite | null): void {
    currentSuite = suite;
}

export function incrementPassed(): void { passed++; }
export function incrementFailed(): void { failed++; }
export function incrementSkipped(): void { skipped++; }
export function addFailure(msg: string): void { failures.push(msg); }

export function reset(): void {
    passed = 0;
    failed = 0;
    skipped = 0;
    failures.length = 0;
}
