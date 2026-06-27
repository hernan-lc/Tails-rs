// ============================================================
// Tails-rs — All Features Example
// Run:  cargo run --bin tails -- examples/all_features.ts
// ============================================================

// --- Variables & Operators ---
console.log("--- Variables & Operators ---");
let x: number = 42;
const pi: number = 3.14;
var label: string = "hello";
let isActive: boolean = true;
console.log(x, pi, label);

// Arithmetic
let sum: number = 10 + 5;
let diff: number = 10 - 3;
let prod: number = 4 * 5;
let div: number = 20 / 4;
let mod: number = 10 % 3;
let pow: number = 2 ** 10;
console.log("arith:", sum, diff, prod, div, mod, pow);

// Compound assignment
let a: number = 10;
a += 5; a -= 3; a *= 2; a /= 4; a %= 3;
console.log("compound:", a);

// Comparison
console.log("eq:", 5 == 5, 5 === 5, 5 != 3, 5 !== "5");

// Logical
console.log("logic:", true && false, true || false, !true);

// Increment / Decrement
let counter: number = 0;
counter++; counter++; counter++;
console.log("counter:", counter);

console.log("pow:", 2 ** 8);
console.log("bitwise NOT:", ~0);

// --- Control Flow ---
console.log("\n--- Control Flow ---");

let score: number = 85;
if (score >= 90) {
    console.log("grade: A");
} else if (score >= 80) {
    console.log("grade: B");
} else {
    console.log("grade: C");
}

let sumFor: number = 0;
for (let i: number = 1; i <= 5; i++) {
    sumFor += i;
}
console.log("for sum 1..5:", sumFor);

let w: number = 0;
let sumWhile: number = 0;
while (w < 5) {
    sumWhile += w;
    w++;
}
console.log("while sum 0..4:", sumWhile);

let dw: number = 0;
let sumDW: number = 0;
do {
    sumDW += dw;
    dw++;
} while (dw < 5);
console.log("do-while sum 0..4:", sumDW);

let day: number = 3;
let dayName: string = "";
switch (day) {
    case 1: dayName = "Mon"; break;
    case 2: dayName = "Tue"; break;
    case 3: dayName = "Wed"; break;
    case 4: dayName = "Thu"; break;
    case 5: dayName = "Fri"; break;
    default: dayName = "Weekend";
}
console.log("day:", dayName);

let age: number = 20;
let status: string = age >= 18 ? "adult" : "minor";
console.log("ternary:", status);

let breakSum: number = 0;
for (let i: number = 0; i < 10; i++) {
    if (i === 5) break;
    breakSum += i;
}
console.log("break at 5:", breakSum);

let continueSum: number = 0;
for (let i: number = 0; i < 10; i++) {
    if (i % 2 === 0) continue;
    continueSum += i;
}
console.log("odd sum:", continueSum);

// for...in
let objKeys: { a: number, b: number, c: number } = { a: 1, b: 2, c: 3 };
let keyCount: number = 0;
for (let key: string in objKeys) {
    keyCount++;
}
console.log("for-in keys:", keyCount);

// --- Interfaces ---
console.log("\n--- Interfaces ---");

interface Point {
    x: number;
    y: number;
}
let origin: Point = { x: 0, y: 0 };
console.log("interface:", origin.x, origin.y);

interface OptionalUser {
    name: string;
    age?: number;
}
let user1: OptionalUser = { name: "Alice" };
let user2: OptionalUser = { name: "Bob", age: 30 };
console.log("optional:", user1.name, user2.name, user2.age);

// --- Functions & Closures ---
console.log("\n--- Functions & Closures ---");

function add(a: number, b: number): number {
    return a + b;
}
console.log("add(3,7):", add(3, 7));

const multiply = function(a: number, b: number): number {
    return a * b;
};
console.log("multiply(4,5):", multiply(4, 5));

const double = function(x: number): number { return x * 2; };
const square = function(x: number): number { return x * x; };
console.log("function expr:", double(7), square(5));

function makeCounter(): any {
    let count: number = 0;
    return function(): any {
        count++;
        return count;
    };
}
const ctr: any = makeCounter();
console.log("closure:", ctr(), ctr(), ctr());

function applyTwice(fn: any, val: number): any {
    return fn(fn(val));
}
console.log("higher-order:", applyTwice(function(x: number): any { return x + 10; }, 5));

// Type alias
type ID = string | number;
let id1: ID = "abc";
let id2: ID = 42;
console.log("type alias:", id1, id2);

// --- Classes ---
console.log("\n--- Classes ---");

class Calculator {
    constructor(a, b) {
        this.a = a;
        this.b = b;
    }
    add() {
        return this.a + this.b;
    }
    mul() {
        return this.a * this.b;
    }
    static create(a: number, b: number): any {
        return new Calculator(a, b);
    }
}
let calc: any = Calculator.create(3, 4);
console.log("class:", calc.add(), calc.mul());

// Inheritance
class Animal {
    constructor(name) {
        this.name = name;
    }
    speak() {
        return this.name + " makes a sound";
    }
}
class Dog extends Animal {
    constructor(name) {
        super(name);
    }
    speak() {
        return this.name + " barks";
    }
    fetch() {
        return this.name + " fetches the ball";
    }
}
let rex: Dog = new Dog("Rex");
console.log("inheritance:", rex.speak(), rex.fetch());
console.log("instanceof:", rex instanceof Dog, rex instanceof Animal);

// Getter and Setter
class Person {
    constructor(first, last) {
        this._first = first;
        this._last = last;
    }
    get fullName(): string {
        return this._first + " " + this._last;
    }
    set fullName(v: string) {
        let parts: string[] = v.split(" ");
        this._first = parts[0];
        this._last = parts[1];
    }
}
let p: Person = new Person("John", "Doe");
console.log("getter:", p.fullName);
p.fullName = "Jane Smith";
console.log("setter:", p.fullName);

// Class expressions
var AnonClass = class {
    constructor() {
        this.msg = "anonymous";
    }
    greet() {
        return this.msg;
    }
};
let anonInst: any = new AnonClass();
console.log("class expr:", anonInst.greet());

// Multiple instances
class Counter {
    constructor(start) {
        this.count = start;
    }
    inc() {
        this.count++;
        return this.count;
    }
}
var ca: Counter = new Counter(0);
var cb: Counter = new Counter(10);
ca.inc(); ca.inc(); ca.inc();
cb.inc();
console.log("multi instance:", ca.count, cb.count);

// Constructor with access modifiers
class Greeter {
    constructor(public greeting: string) {}
    greet(): string {
        return this.greeting;
    }
}
const greeter: Greeter = new Greeter("Hello!");
console.log("constructor modifiers:", greeter.greet());

// --- Promises ---
console.log("\n--- Promises ---");

var pResult: number = 0;
Promise.resolve(99).then(function(v: number): void { pResult = v; });
console.log("Promise.resolve:", pResult);

var pErr: string = "";
Promise.reject("err").catch(function(v: string): void { pErr = v; });
console.log("Promise.catch:", pErr);

var chainResult: number = 0;
Promise.resolve(10)
    .then(function(v: number): number { return v + 5; })
    .then(function(v: number): void { chainResult = v; });
console.log("chain:", chainResult);

var allResult: number = 0;
Promise.all([Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)])
    .then(function(arr: any[]): void { allResult = arr.length; });
console.log("Promise.all:", allResult);

var finallyResult: number = 0;
Promise.resolve(1)
    .then(function(v: number): void { finallyResult += v; })
    .finally(function(): void { finallyResult += 10; });
console.log("finally:", finallyResult);

var ctorResult: number = 0;
var ctorPromise: any = new Promise(function(resolve: any): void { resolve(42); });
ctorPromise.then(function(v: any): void { ctorResult = v; });
console.log("constructor:", ctorResult);

// --- Timers ---
console.log("\n--- Timers ---");

var timerResult: string = "not fired";
setTimeout(function(): void {
    timerResult = "fired";
}, 0);
console.log("setTimeout registered:", timerResult);

var intervalResult: number = 0;
var id: number = setInterval(function(): void {
    intervalResult++;
}, 10);
clearInterval(id);
console.log("setInterval/clear:", intervalResult);

// --- JSON ---
console.log("\n--- JSON ---");

let jsonStr: string = JSON.stringify({ name: "Alice", age: 30 });
console.log("stringify:", jsonStr);

let parsed: any = JSON.parse('{"x": 10, "y": "hello"}');
console.log("parse:", parsed.x, parsed.y);

let roundTrip: any = JSON.parse(JSON.stringify({ a: 1, b: 2 }));
console.log("roundtrip:", roundTrip.a, roundTrip.b);

// --- Object Methods ---
console.log("\n--- Object Methods ---");

let obj: any = { a: 1, b: 2, c: 3 };
console.log("keys:", Object.keys(obj).length);
console.log("values:", Object.values(obj).length);
console.log("entries:", Object.entries(obj).length);

let target: any = { a: 1 };
Object.assign(target, { b: 2 }, { c: 3 });
console.log("assign:", target.a, target.b, target.c);

// --- Array Methods ---
console.log("\n--- Array Methods ---");

let pushArr: number[] = [1, 2];
pushArr.push(3);
console.log("push:", pushArr.length);

let popArr: number[] = [1, 2, 3];
popArr.pop();
console.log("pop:", popArr.length);

let shiftArr: number[] = [1, 2, 3];
let shifted: number = shiftArr.shift();
console.log("shift:", shifted, shiftArr.length);

let unshiftArr: number[] = [2, 3];
unshiftArr.unshift(1);
console.log("unshift:", unshiftArr.length);

let mapped: number[] = [1, 2, 3].map(function(x: number): number { return x * 2; });
console.log("map:", mapped[0], mapped[1], mapped[2]);

let evens: number[] = [1, 2, 3, 4, 5, 6].filter(function(x: number): boolean { return x % 2 === 0; });
console.log("filter:", evens.length);

let total: number = [1, 2, 3, 4].reduce(function(acc: number, x: number): number { return acc + x; }, 0);
console.log("reduce:", total);

let found: any = [10, 20, 30].find(function(x: number): boolean { return x > 15; });
let foundIdx: number = [10, 20, 30].findIndex(function(x: number): boolean { return x > 15; });
console.log("find:", found, foundIdx);

let hasBig: boolean = [1, 2, 3].some(function(x: number): boolean { return x > 2; });
let allPos: boolean = [1, 2, 3].every(function(x: number): boolean { return x > 0; });
console.log("some/every:", hasBig, allPos);

let idx: number = [10, 20, 30].indexOf(20);
let inc: boolean = [1, 2, 3].includes(2);
console.log("indexOf/includes:", idx, inc);

let joined: string = ["a", "b", "c"].join("-");
console.log("join:", joined);

let reversed: number[] = [1, 2, 3].reverse();
let sorted: number[] = [3, 1, 4, 1, 5].sort();
console.log("reverse:", reversed[0], "sort:", sorted[0]);

let concat: number[] = [1, 2].concat([3, 4]);
console.log("concat:", concat.length);

let sliced: number[] = [1, 2, 3, 4, 5].slice(1, 3);
console.log("slice:", sliced.length);

let spliced: number[] = [1, 2, 3, 4];
let removed: number[] = spliced.splice(1, 2);
console.log("splice:", removed.length, spliced.length);

let flat: number[] = [[1, 2], [3, 4]].flat();
console.log("flat:", flat.length);

let feSum: number = 0;
[1, 2, 3].forEach(function(x: number): void { feSum += x; });
console.log("forEach:", feSum);

// --- String Methods ---
console.log("\n--- String Methods ---");

console.log("charAt:", "hello".charAt(1));
console.log("charCodeAt:", "A".charCodeAt(0));
console.log("slice:", "hello".slice(1, 3));
console.log("substring:", "hello".substring(1, 4));
console.log("indexOf:", "hello world".indexOf("world"));
console.log("includes:", "hello".includes("ell"));
console.log("replace:", "hello world".replace("world", "TS"));
console.log("split:", "a,b,c".split(",").length);
console.log("trim:", "  hello  ".trim());
console.log("toLowerCase:", "HELLO".toLowerCase());
console.log("toUpperCase:", "hello".toUpperCase());
console.log("startsWith:", "hello".startsWith("hel"));
console.log("endsWith:", "hello".endsWith("llo"));
console.log("repeat:", "ha".repeat(3));
console.log("padStart:", "5".padStart(3, "0"));
console.log("padEnd:", "5".padEnd(3, "0"));

// --- Math ---
console.log("\n--- Math ---");

console.log("PI:", Math.PI);
console.log("E:", Math.E);
console.log("abs:", Math.abs(-5));
console.log("floor:", Math.floor(3.7));
console.log("ceil:", Math.ceil(3.2));
console.log("round:", Math.round(3.5));
console.log("min:", Math.min(3, 1, 4, 1, 5));
console.log("max:", Math.max(3, 1, 4, 1, 5));
console.log("pow:", Math.pow(2, 10));
console.log("sqrt:", Math.sqrt(16));
console.log("log:", Math.log(1));
console.log("sin:", Math.sin(0));
console.log("cos:", Math.cos(0));
console.log("tan:", Math.tan(0));

let rand: number = Math.random();
console.log("random 0-1:", rand >= 0.0 && rand <= 1.0);

// --- Error Handling ---
console.log("\n--- Error Handling ---");

try {
    let val: number = 10;
    console.log("try:", val);
} catch (e: any) {
    console.log("catch (should not reach)");
} finally {
    console.log("finally: done");
}

try {
    throw "custom error";
} catch (e: any) {
    console.log("caught:", e);
}

// --- Template Literals ---
console.log("\n--- Template Literals ---");

let name: string = "World";
let greeting: string = `Hello ${name}!`;
console.log(greeting);

let v: number = 10;
let msg: string = `The value is ${v * 2}`;
console.log(msg);

let a2: string = "Hello";
let b2: string = "World";
let combined: string = `${a2}, ${b2}!`;
console.log(combined);

// --- Global Functions ---
console.log("\n--- Global Functions ---");

console.log("parseInt:", parseInt("42"));
console.log("parseInt negative:", parseInt("-42"));
console.log("parseFloat:", parseFloat("3.14"));
console.log("isNaN NaN:", isNaN(NaN));
console.log("isNaN 42:", isNaN(42));
console.log("isFinite:", isFinite(42));

// Number methods
console.log("Number.parseInt:", Number.parseInt("42"));
console.log("Number.parseFloat:", Number.parseFloat("3.14"));
console.log("Number.isNaN:", Number.isNaN(NaN));
console.log("Number.isFinite:", Number.isFinite(42));

// --- Destructuring ---
console.log("\n--- Destructuring ---");

let [da, db, dc]: number[] = [10, 20, 30];
console.log("array destr:", da, db, dc);

let { x: ox, y: oy }: { x: number, y: number } = { x: 100, y: 200 };
console.log("obj destr:", ox, oy);

let [d1, , d3]: number[] = [1, 2, 3];
console.log("skip:", d1, d3);

// --- Spread ---
console.log("\n--- Spread ---");

let sa: number[] = [1, 2, 3];
let sb: number[] = [4, 5, 6];
let sc: number[] = [...sa, ...sb];
console.log("spread arr:", sc.length);

// --- Type Checks ---
console.log("\n--- Type Checks ---");

console.log("typeof 42:", typeof 42);
console.log("typeof hello:", typeof "hello");
console.log("typeof true:", typeof true);
console.log("typeof undefined:", typeof undefined);
console.log("typeof null:", typeof null);
console.log("void:", typeof (void 0));

// --- instanceof & in ---
console.log("\n--- instanceof & in ---");

class Foo {}
let foo: Foo = new Foo();
console.log("instanceof Foo:", foo instanceof Foo);

let inObj: { x: number, y: number } = { x: 1, y: 2 };
console.log("in operator:", "x" in inObj, "z" in inObj);

// --- Nullish & Identity ---
console.log("\n--- Identity ---");

console.log("null == null:", null == null);
console.log("null === null:", null === null);
console.log("undefined == undefined:", undefined == undefined);
console.log("NaN === NaN:", NaN === NaN);

console.log("\n========== ALL FEATURES DONE ==========");
