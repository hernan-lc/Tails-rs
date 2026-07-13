export type TestFn = () => void | Promise<void>;
export type HookFn = () => void | Promise<void>;

export interface TestCase {
    name: string;
    fn: TestFn;
    skip: boolean;
}

export interface TestSuite {
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
