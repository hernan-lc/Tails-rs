import { TestSuite } from './types.ts';
import { passed, failed, skipped, failures, reset, rootSuites } from './state.ts';

async function runSuite(suite: TestSuite, prefix: string): Promise<void> {
    const fullName = prefix ? `${prefix} > ${suite.name}` : suite.name;
    if (suite.name) console.log(`\n📦 ${fullName}`);

    for (const hook of suite.beforeAll) {
        try { await hook(); } catch (e) {
            const s = e instanceof Error ? e.message : String(e);
            failures.push(`[beforeAll in "${fullName}"] ${s}`);
            failed += suite.tests.length;
            return;
        }
    }

    for (const test of suite.tests) {
        if (test.skip) { console.log(`  ⏭️  SKIPPED: ${test.name}`); skipped++; continue; }

        let beforeEachFailed = false;
        let beforeEachError = '';
        for (const hook of suite.beforeEach) {
            try { await hook(); } catch (e) {
                beforeEachFailed = true;
                beforeEachError = e instanceof Error ? e.message : String(e);
                break;
            }
        }

        if (beforeEachFailed) {
            console.log(`  ❌ ${test.name} (beforeEach failed)`);
            failures.push(`[${fullName}] ${test.name}: ${beforeEachError}`);
            failed++;
            continue;
        }

        const start = Date.now();
        try {
            await test.fn();
            const elapsed = Date.now() - start;
            console.log(`  ✅ ${test.name} (${elapsed}ms)`);
            passed++;
        } catch (e) {
            const elapsed = Date.now() - start;
            const s = e instanceof Error ? e.message : String(e);
            console.log(`  ❌ ${test.name} (${elapsed}ms)`);
            console.log(`     ${s}`);
            failures.push(`[${fullName}] ${test.name}: ${s}`);
            failed++;
        }

        for (const hook of suite.afterEach) {
            try { await hook(); } catch (e) {
                const s = e instanceof Error ? e.message : String(e);
                console.log(`     ⚠️  afterEach failed: ${s}`);
            }
        }
    }

    for (const child of suite.children) await runSuite(child, fullName);

    for (const hook of suite.afterAll) {
        try { await hook(); } catch (e) {
            const s = e instanceof Error ? e.message : String(e);
            console.log(`  ⚠️  [afterAll] ${s}`);
        }
    }
}

export async function runTests(): Promise<{ passed: number; failed: number; skipped: number }> {
    reset();
    console.log('\n🧪 Running tests...\n');
    for (const suite of rootSuites) await runSuite(suite, '');
    console.log('\n' + '='.repeat(50));
    console.log(`Results: ${passed} passed, ${failed} failed, ${skipped} skipped`);
    console.log('='.repeat(50));
    if (failures.length > 0) {
        console.log('\nFailures:');
        failures.forEach((f, i) => console.log(`  ${i + 1}. ${f}`));
    }
    return { passed, failed, skipped };
}
