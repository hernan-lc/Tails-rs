// ============================================================
// Tails-rs — math_utils.ts
// A reusable module exporting common math helpers.
// ============================================================

export function add(a: number, b: number): number {
    return a + b;
}

export function subtract(a: number, b: number): number {
    return a - b;
}

export function multiply(a: number, b: number): number {
    return a * b;
}

export function square(x: number): number {
    return x * x;
}

export const PI: number = 3.141592653589793;
export const E: number = 2.718281828459045;

export default add;
