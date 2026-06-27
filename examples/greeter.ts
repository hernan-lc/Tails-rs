// ============================================================
// Tails-rs — greeter.ts
// A small module exporting a Greeter class and a helper.
// ============================================================

export class Greeter {
    constructor(private greeting: string) {}

    greet(name: string): string {
        return `${this.greeting}, ${name}!`;
    }
}

export function shout(text: string): string {
    return text.toUpperCase() + "!";
}

export default Greeter;
