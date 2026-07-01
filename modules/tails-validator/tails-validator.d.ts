// tails-validator — native validation module
// Schema-based validation with zero-copy NativeValue handles

/** Opaque handle to a native value stored in the handle registry. */
export type NativeValue = unknown;

/** Schema handle — an opaque NativeValue pointing to a schema definition. */
export type Schema = NativeValue;

/** A single validation issue returned on failure. */
export interface ValidationIssue {
  code: string;
  message: string;
  path?: Array<{ key: string }>;
  expected?: string;
  received?: string;
}

/** The error payload when validation fails. */
export interface ValidationError {
  issues: ValidationIssue[];
}

/** Successful validation result. */
export interface ValidationOk {
  success: true;
  data: unknown;
}

/** Failed validation result. */
export interface ValidationErr {
  success: false;
  error: ValidationError;
}

/** Discriminated union of validation results. */
export type ValidateResult = ValidationOk | ValidationErr;

// ── Core ────────────────────────────────────────────────────────────────

/** Validate a value against a schema. Returns a JSON string of the ValidateResult. */
export function validate(schema: Schema, value: unknown): string;

/** Format a validation error into a human-readable string. */
export function format_error(error: ValidationError): string;

// ── Base type builders ──────────────────────────────────────────────────

export function z(): Schema;
export function string(): Schema;
export function number(): Schema;
export function boolean(): Schema;
export function nil(): Schema;
export function any(): Schema;
export function unknown(): Schema;

// ── String validators ───────────────────────────────────────────────────

export function stringMin(min: number): Schema;
export function stringMax(max: number): Schema;
export function stringLength(len: number): Schema;
export function stringPattern(pattern: string): Schema;
export function stringEmail(): Schema;
export function stringUrl(): Schema;
export function stringUuid(): Schema;
export function stringDatetime(): Schema;
export function stringIPv4(): Schema;
export function stringIPv6(): Schema;
export function stringPhone(): Schema;
export function stringBase64(): Schema;

// ── Number validators ───────────────────────────────────────────────────

export function numberMin(min: number): Schema;
export function numberMax(max: number): Schema;
export function numberInt(): Schema;
export function numberPositive(): Schema;
export function numberNegative(): Schema;
export function numberMultipleOf(n: number): Schema;
export function numberFinite(): Schema;

// ── Array validators ────────────────────────────────────────────────────

export function arrayMin(itemsSchema: Schema, min: number): Schema;
export function arrayMax(itemsSchema: Schema, max: number): Schema;
export function arrayLength(itemsSchema: Schema, len: number): Schema;
export function arrayUnique(itemsSchema: Schema): Schema;

// ── Composable validators ───────────────────────────────────────────────

export function optional(innerSchema: Schema): Schema;
export function nullable(innerSchema: Schema): Schema;
export function transform(innerSchema: Schema, transformName: string): Schema;
export function refine(innerSchema: Schema, message: string): Schema;
export function pipe(schemas: Schema[]): Schema;
export function preprocess(transformName: string, innerSchema: Schema): Schema;
export function withDefault(innerSchema: Schema, defaultValue: unknown): Schema;
export function customError(innerSchema: Schema, message: string): Schema;
export function lazy(id: string, innerSchema: Schema): Schema;
export function literal(value: unknown): Schema;
export function enumValues(values: unknown[]): Schema;
export function object(
  properties: Record<string, Schema>,
  required: string[],
  strict: boolean,
): Schema;
export function record(valuesSchema: Schema): Schema;
export function union(schemas: Schema[]): Schema;
export function intersection(schemas: Schema[]): Schema;
export function tuple(schemas: Schema[]): Schema;
export function coerce(target: string, innerSchema: Schema): Schema;
