// ============================================================
// Tails-rs — All Features Demo
// Run: cargo run --bin tails -- examples/all_features.ts
//
// Comprehensive test of every feature listed in the README.
// ============================================================
import path from "./path.native";
import fs from "./fs.native";
import process from "./process.native";
import Buffer from "./buffer.native";
import Intl from "./intl.native";

console.log("=== Tails-rs Feature Demo ===\n");

// --- Variables & Types ---
console.log("--- Variables & Types ---");
let x = 10;
const y = 20;
var z = 30;
console.log("let x:", x, "const y:", y, "var z:", z);

let s = "hello";
let b = true;
let n = null;
let u = undefined;
console.log("string:", s, "boolean:", b, "null:", n, "undefined:", u);

// Operators
console.log("arithmetic:", 10 + 3, 10 - 3, 10 * 3, 10 / 3, 10 % 3, 2 ** 10);
x += 5;
console.log("compound:", x);
console.log("comparison:", 5 == 5, 5 === 5, 5 != 3, 5 !== "5", 5 < 10, 5 > 3);
console.log("logical:", true && false, true || false, !true);
console.log("typeof:", typeof 42, typeof "hi", typeof true, typeof undefined);
console.log("void:", typeof void 0);

// Increment/Decrement
let counter = 0;
counter++;
counter++;
console.log("++:", counter);
counter--;
console.log("--:", counter);

// --- Control Flow ---
console.log("\n--- Control Flow ---");
let a = 5;
if (a > 10) {
  console.log("big");
} else if (a > 3) {
  console.log("medium");
} else {
  console.log("small");
}
let ternary = a > 3 ? "big" : "small";
console.log("ternary:", ternary);

// for loop
let sum = 0;
for (let i = 1; i <= 5; i++) {
  sum += i;
}
console.log("for sum:", sum);

// while
let w = 5;
while (w > 0) {
  w--;
}
console.log("while done:", w);

// do while
let d = 0;
do {
  d++;
} while (d < 3);
console.log("do while:", d);

// for...in
let obj = { a: 1, b: 2 };
let keys = "";
for (let k in obj) {
  keys += k;
}
console.log("for...in keys:", keys);

// switch
let day = 2;
let dayName = "";
switch (day) {
  case 1:
    dayName = "Mon";
    break;
  case 2:
    dayName = "Tue";
    break;
  default:
    dayName = "Other";
}
console.log("switch:", dayName);

// --- Functions ---
console.log("\n--- Functions ---");
function add(a: number, b: number): number {
  return a + b;
}
console.log("function:", add(3, 4));

let mul = (a: number, b: number): number => a * b;
console.log("arrow:", mul(3, 4));

function makeCounter() {
  let count = 0;
  return function () {
    count++;
    return count;
  };
}
let counterFn = makeCounter();
console.log("closure:", counterFn(), counterFn(), counterFn());

let applyFn = (fn: Function, x: number, y: number) => fn(x, y);
console.log("higher-order:", applyFn(add, 10, 20));

// --- Classes ---
console.log("\n--- Classes ---");
class Animal {
  name: string;
  constructor(name: string) {
    this.name = name;
  }
  speak(): string {
    return this.name + " makes a noise";
  }
  get label(): string {
    return "animal:" + this.name;
  }
  set label(v: string) {
    this.name = v.split(":")[1];
  }
}

class Dog extends Animal {
  speak(): string {
    return this.name + " barks";
  }
}

let d2 = new Dog("Rex");
console.log("class:", d2.speak());
console.log("getter:", d2.label);
d2.label = "animal:Spot";
console.log("setter:", d2.speak());
console.log("instanceof:", d2 instanceof Animal, d2 instanceof Dog);

// Static methods
class MathHelper {
  static double(x: number): number {
    return x * 2;
  }
}
console.log("static:", MathHelper.double(5));

// --- Objects & Arrays ---
console.log("\n--- Objects & Arrays ---");
let person = { name: "Alice", age: 30 };
console.log("Object.keys:", Object.keys(person));
console.log("Object.values:", Object.values(person));
console.log("Object.entries:", Object.entries(person).length);
Object.assign(person, { city: "NYC" });
console.log("Object.assign:", person.city);

let arr = [1, 2, 3, 4, 5];
console.log("push:", arr.push(6));
console.log("pop:", arr.pop());
console.log(
  "map:",
  arr.map((x: number) => x * 2)
);
console.log(
  "filter:",
  arr.filter((x: number) => x > 3)
);
console.log(
  "reduce:",
  arr.reduce((a: number, b: number) => a + b, 0)
);
console.log(
  "find:",
  arr.find((x: number) => x > 3)
);
console.log(
  "some:",
  arr.some((x: number) => x > 4)
);
console.log(
  "every:",
  arr.every((x: number) => x > 0)
);
console.log("includes:", arr.includes(3));
console.log("join:", arr.join("-"));
console.log("slice:", arr.slice(1, 3));
console.log(
  "flat:",
  [
    [1, 2],
    [3, 4]
  ].flat()
);

// --- Typed Arrays ---
console.log("\n--- Typed Arrays ---");
let ta = new Int32Array(3);
ta.set(0, 10);
ta.set(1, 20);
ta.set(2, 30);
console.log("Int32Array length:", ta.length);
console.log("Int32Array get:", ta.get(0), ta.get(1), ta.get(2));

let ta2 = new Float64Array([1.5, 2.5, 3.5]);
console.log("Float64Array length:", ta2.length);
console.log("Float64Array get:", ta2.get(0), ta2.get(1), ta2.get(2));

// --- ES6+ Collections ---
console.log("\n--- ES6+ Collections ---");
let myMap = new Map();
myMap.set("a", 1);
myMap.set("b", 2);
myMap.set("c", 3);
console.log("Map size:", myMap.size);
console.log("Map get:", myMap.get("a"), myMap.get("b"));
console.log("Map has:", myMap.has("a"), myMap.has("d"));
myMap.delete("c");
console.log("Map after delete:", myMap.size);

let mySet = new Set();
mySet.add(1);
mySet.add(2);
mySet.add(3);
mySet.add(2);
console.log("Set size:", mySet.size);
console.log("Set has:", mySet.has(1), mySet.has(4));
mySet.delete(1);
console.log("Set after delete:", mySet.size);
// --- Strings ---
console.log("\n--- Strings ---");
let str = "Hello, World!";
console.log("charAt:", str.charAt(0));
console.log("charCodeAt:", str.charCodeAt(0));
console.log("slice:", str.slice(0, 5));
console.log("substring:", str.substring(7, 12));
console.log("indexOf:", str.indexOf("World"));
console.log("includes:", str.includes("World"));
console.log("replace:", str.replace("World", "Tails"));
console.log("split:", str.split(", "));
console.log("trim:", "  hi  ".trim());
console.log("toLowerCase:", str.toLowerCase());
console.log("toUpperCase:", str.toUpperCase());
console.log("startsWith:", str.startsWith("Hello"));
console.log("endsWith:", str.endsWith("World!"));
console.log("padStart:", "5".padStart(3, "0"));
console.log("repeat:", "ab".repeat(3));

// --- Math ---
console.log("\n--- Math ---");
console.log("PI:", Math.PI);
console.log("E:", Math.E);
console.log("abs:", Math.abs(-5));
console.log("floor:", Math.floor(3.7));
console.log("ceil:", Math.ceil(3.2));
console.log("round:", Math.round(3.5));
console.log("min/max:", Math.min(1, 2, 3), Math.max(1, 2, 3));
console.log("pow:", Math.pow(2, 10));
console.log("sqrt:", Math.sqrt(16));
console.log("sin:", Math.sin(0));
console.log("random:", Math.random() >= 0);

// --- JSON ---
console.log("\n--- JSON ---");
let json = JSON.stringify({ a: 1, b: [2, 3] });
console.log("stringify:", json);
let parsed = JSON.parse(json);
console.log("parse:", parsed.a, parsed.b.length);

// --- Error Handling ---
console.log("\n--- Error Handling ---");
try {
  throw new Error("test error");
} catch (e) {
  console.log("catch:", e.message);
}

try {
  throw new TypeError("bad type");
} catch (e) {
  console.log("TypeError:", e.name, e.message);
  console.log("stack:", typeof e.stack === "string");
}

// --- Global Functions ---
console.log("\n--- Global Functions ---");
console.log("parseInt:", parseInt("42"), parseInt("0xFF"));
console.log("parseFloat:", parseFloat("3.14"));
console.log("isNaN:", isNaN(NaN), isNaN(42));
console.log("isFinite:", isFinite(42), isFinite(Infinity));

// --- Encoding ---
console.log("\n--- Encoding ---");
let encoded = btoa("Hello, World!");
console.log("btoa:", encoded);
console.log("atob:", atob(encoded));

// --- Buffer ---
console.log("\n--- Buffer ---");
let buf = Buffer.from("Hello");
console.log("Buffer from:", buf.toString());
console.log("Buffer length:", buf.length);
let buf2 = Buffer.alloc(5, 65);
console.log("Buffer alloc:", buf2.toString());
let buf3 = Buffer.concat([buf, Buffer.from(" World")]);
console.log("Buffer concat:", buf3.toString());
console.log("Buffer isBuffer:", Buffer.isBuffer(buf));

// --- process ---
console.log("\n--- process ---");
console.log("platform:", process.platform);
console.log("arch:", process.arch);
console.log("pid:", typeof process.pid === "number");
console.log("cwd:", typeof process.cwd() === "string");
console.log("env:", typeof process.env === "object");

// --- path ---
console.log("\n--- path ---");
console.log("join:", path.join("/foo", "bar", "baz"));
console.log("basename:", path.basename("/foo/bar.txt"));
console.log("dirname:", path.dirname("/foo/bar.txt"));
console.log("extname:", path.extname("/foo/bar.txt"));
console.log("isAbsolute:", path.isAbsolute("/foo"));
console.log("normalize:", path.normalize("/foo/../bar"));
console.log("sep:", path.sep === "/" || path.sep === "\\");

// --- fs ---
console.log("\n--- fs ---");
fs.writeFileSync("/tmp/tails_test.txt", "Hello from Tails!");
console.log("readFileSync:", fs.readFileSync("/tmp/tails_test.txt"));
console.log("existsSync:", fs.existsSync("/tmp/tails_test.txt"));
fs.unlinkSync("/tmp/tails_test.txt");
console.log("after delete:", fs.existsSync("/tmp/tails_test.txt"));

// --- Destructuring & Spread ---
console.log("\n--- Destructuring & Spread ---");
let [p, q, ...rest] = [1, 2, 3, 4, 5];
console.log("array destruct:", p, q, rest);

let { name: n2, age } = { name: "Bob", age: 25 };
console.log("object destruct:", n2, age);

let arr2 = [1, 2, 3];
let arr3 = [...arr2, 4, 5];
console.log("spread:", arr3);

// --- Proxy ---
console.log("\n--- Proxy ---");
let handler = {
  get: function (target: any, prop: string) {
    return prop in target ? target[prop] : 42;
  },
};
let proxy = new Proxy({ x: 1 }, handler);
console.log("Proxy get:", proxy.x);
console.log("Proxy missing:", proxy.missing);

// --- Optional chaining & nullish coalescing ---
console.log("\n--- Optional Chaining ---");
let user: any = { address: { city: "NYC" } };
console.log("optional:", user?.address?.city);
console.log("nullish:", null ?? "default");
console.log("nullish 2:", 0 ?? "default");

// --- Symbol ---
console.log("\n--- Symbol ---");
let sym = Symbol("test");
console.log("Symbol:", typeof sym);
let sym2 = Symbol.for("test");
console.log("Symbol.for:", typeof sym2);
console.log("Symbol.keyFor:", Symbol.keyFor(sym2));

// --- for...of ---
console.log("\n--- for...of ---");
let arr4 = [10, 20, 30];
let sum2 = 0;
for (let v of arr4) {
  sum2 += v;
}
console.log("for...of sum:", sum2);

let str2 = "abc";
let chars = "";
for (let c of str2) {
  chars += c + "-";
}
console.log("string for...of:", chars);

// --- Generator ---
console.log("\n--- Generator ---");
function* idGen() {
  let id = 0;
  while (true) {
    yield id++;
  }
}
let gen = idGen();
console.log("generator:", gen.next().value, gen.next().value, gen.next().value);

// --- Promise & Async ---
console.log("\n--- Promise & Async ---");
let p2 = Promise.resolve(42);
p2.then((v: number) => console.log("Promise resolve:", v));

Promise.all([Promise.resolve(1), Promise.resolve(2)]).then((vals: number[]) => {
  console.log("Promise.all:", vals);
});

// --- BigInt ---
console.log("\n--- BigInt ---");
let big = 42n;
let big2 = BigInt(10);
console.log("BigInt:", typeof big, big + big2);

// --- Date ---
console.log("\n--- Date ---");
let now = new Date();
console.log("Date:", typeof now.getTime() === "number");
console.log("Date.now:", typeof Date.now() === "number");
console.log("getFullYear:", typeof now.getFullYear() === "number");

// --- RegExp ---
console.log("\n--- RegExp ---");
// Note: Regex literals not supported in parser, use new RegExp()
let re2 = new RegExp("\\d+", "g");
console.log("RegExp test:", re2.test("abc123"));

// --- Iterator Helpers ---
console.log("\n--- Iterator Helpers ---");
let iterArr = [1, 2, 3, 4, 5];
// Note: iterator helpers require Symbol.iterator support on arrays

// --- Function.prototype ---
console.log("\n--- Function.prototype ---");
function greet(greeting: string, name: string) {
  return greeting + ", " + name + "!";
}
console.log("call:", greet.call(null, "Hi", "World"));
console.log("apply:", greet.apply(null, ["Hey", "Alice"]));
let bound = greet.bind(null, "Hello");
console.log("bind:", bound("Bob"));

// --- Object freeze/seal ---
console.log("\n--- Object freeze/seal ---");
let frozen = Object.freeze({ x: 1 });
console.log("isFrozen:", Object.isFrozen(frozen));
let sealed = Object.seal({ y: 2 });
console.log("isSealed:", Object.isSealed(sealed));
console.log("is:", Object.is(1, 1), Object.is(NaN, NaN));

// --- Intl ---
console.log("\n--- Intl ---");
let dtf = new Intl.DateTimeFormat("en-US", {
  year: "numeric",
  month: "long",
  day: "numeric",
});
console.log("DateTimeFormat:", dtf.format().length > 0);
let nf = new Intl.NumberFormat("en-US");
console.log("NumberFormat:", nf.format(1234567.89));

// --- URL ---
console.log("\n--- URL ---");
let url = new URL("https://example.com/path?foo=bar&baz=qux");
console.log("URL href:", url.href);
console.log("URL protocol:", url.protocol);
console.log("URL searchParams get:", url.searchParams.get("foo"));

console.log("\n=== ALL FEATURES DEMO COMPLETE ===");
