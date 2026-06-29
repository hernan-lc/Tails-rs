// ============================================================================
// 1. Standard Schema v1 Definitions & Utility Types
// ============================================================================

export interface StandardIssue {
  readonly message: string;
  readonly path?: ReadonlyArray<{ readonly key: string | number | symbol }>;
}

export interface StandardResult<T> {
  readonly value?: T;
  readonly issues?: ReadonlyArray<StandardIssue>;
}

export interface StandardSchemaV1<Input = unknown, Output = Input> {
  readonly "~standard": {
    readonly version: 1;
    readonly vendor: string;
    readonly validate: (
      value: unknown,
    ) => StandardResult<Output> | Promise<StandardResult<Output>>;
  };
}

export type InferInput<T extends Schema<any, any>> = T["_input"];
export type InferOutput<T extends Schema<any, any>> = T["_output"];

export interface Dataset<T> {
  value: T;
  typed: boolean;
  issues?: any[];
}

export interface Schema<Input, Output> extends StandardSchemaV1<Input, Output> {
  kind: "schema" | "validation";
  type: string;
  reference: (...args: any[]) => any;
  expects: string;
  async: boolean;
  message?: string;
  _input: Input;
  _output: Output;
  "~run": (dataset: Dataset<any>, config: any) => Dataset<any>;
}

// Helper to construct structural issue outputs
const createIssue = (context: any, received: unknown, message?: string) => ({
  kind: context.kind,
  type: context.type,
  input: received,
  expected: context.expects,
  received: typeof received,
  message:
    message ||
    `Invalid ${context.type}: Expected ${context.expects} but received ${typeof received}`,
});

// ============================================================================
// 2. Schema Implementations
// ============================================================================

export const stringFn = (msg?: string): Schema<string, string> => ({
  kind: "schema",
  type: "string",
  reference: stringFn,
  expects: "string",
  async: false,
  message: msg,
  get _input() {
    return undefined as any;
  },
  get _output() {
    return undefined as any;
  },
  get "~standard"() {
    return {
      version: 1,
      vendor: "x",
      validate: (v: unknown) => {
        const result = this["~run"]({ value: v, typed: false }, {});
        return result.issues
          ? { issues: result.issues }
          : { value: result.value };
      },
    };
  },
  "~run"(dataset, config) {
    if (typeof dataset.value !== "string") {
      dataset.issues = [createIssue(this, dataset.value, this.message)];
      dataset.typed = false;
    } else {
      dataset.typed = true;
    }
    return dataset;
  },
});

export const minLength = (n: number, msg?: string): Schema<string, string> => ({
  kind: "validation",
  type: "min_length",
  reference: minLength,
  expects: `>=${n}`,
  async: false,
  message: msg,
  get _input() {
    return undefined as any;
  },
  get _output() {
    return undefined as any;
  },
  get "~standard"() {
    return {
      version: 1,
      vendor: "x",
      validate: (v: unknown) => {
        const result = this["~run"]({ value: v, typed: true }, {});
        return result.issues
          ? { issues: result.issues }
          : { value: result.value };
      },
    };
  },
  "~run"(dataset, config) {
    if (
      dataset.typed &&
      typeof dataset.value === "string" &&
      dataset.value.length < n
    ) {
      dataset.issues = dataset.issues || [];
      dataset.issues.push(createIssue(this, dataset.value, this.message));
    }
    return dataset;
  },
});

export const pipe = <
  T extends Schema<any, any>,
  V extends Schema<InferOutput<T>, InferOutput<T>>[],
>(
  base: T,
  ...validations: V
): Schema<InferInput<T>, InferOutput<T>> => ({
  ...base,
  pipe: [base, ...validations],
  get "~standard"() {
    return {
      version: 1,
      vendor: "x",
      validate: (v: unknown) => {
        const result = this["~run"]({ value: v, typed: false }, {});
        return result.issues
          ? { issues: result.issues }
          : { value: result.value };
      },
    };
  },
  "~run"(dataset, config) {
    let currentDataset = dataset;
    for (const action of (this as any).pipe) {
      currentDataset = action["~run"](currentDataset, config);
    }
    return currentDataset;
  },
});

type ObjectEntries = { [key: string]: Schema<any, any> };

type InferObjectOutput<T extends ObjectEntries> = {
  [K in keyof T]: InferOutput<T[K]>;
};
type InferObjectInput<T extends ObjectEntries> = {
  [K in keyof T]: InferInput<T[K]>;
};

export const objectFn = <T extends ObjectEntries>(
  entries: T,
  msg?: string,
): Schema<InferObjectInput<T>, InferObjectOutput<T>> => ({
  kind: "schema",
  type: "object",
  reference: objectFn,
  expects: "Object",
  async: false,
  entries,
  message: msg,
  get _input() {
    return undefined as any;
  },
  get _output() {
    return undefined as any;
  },
  get "~standard"() {
    return {
      version: 1,
      vendor: "x",
      validate: (v: unknown) => {
        const result = this["~run"]({ value: v, typed: false }, {});
        return result.issues
          ? { issues: result.issues }
          : { value: result.value };
      },
    };
  },
  "~run"(dataset, config) {
    const value = dataset.value;

    if (!value || typeof value !== "object" || Array.isArray(value)) {
      dataset.issues = [createIssue(this, value, this.message)];
      dataset.typed = false;
      return dataset;
    }

    const output: Record<string, any> = {};
    let hasIssues = false;
    dataset.issues = dataset.issues || [];

    for (const key in this.entries) {
      const schema = this.entries[key];
      const keyDataset = schema["~run"](
        { value: (value as any)[key], typed: false },
        config,
      );

      if (keyDataset.issues && keyDataset.issues.length > 0) {
        hasIssues = true;
        const mappedIssues = keyDataset.issues.map((issue: any) => ({
          ...issue,
          path: [
            {
              type: "object",
              origin: "value",
              key,
              value: (value as any)[key],
            },
            ...(issue.path || []),
          ],
        }));
        dataset.issues!.push(...mappedIssues);
      }

      output[key] = keyDataset.value;
    }

    dataset.value = output;
    dataset.typed = !hasIssues;
    return dataset;
  },
});

// ============================================================================
// 3. Execution & Type System Verification
// ============================================================================

const userSchema = objectFn({
  username: pipe(
    stringFn("Username must be a string"),
    minLength(3, "Username too short"),
  ),
  email: stringFn("Email is required"),
});

// --- Static Typing Verification ---
// TS compiler automatically infers this structure precisely:
// type ExpectedOutput = { username: string; email: string; }
type UserOutput = InferOutput<typeof userSchema>;

// --- Runtime Execution Debug ---
console.log("--- TEST 1: Invalid Object Types ---");
const badResult = userSchema["~standard"].validate({
  username: "jo", // Fails validation (min length 3)
  email: 42, // Fails basic type check
});
console.log(JSON.stringify(badResult, null, 2));
