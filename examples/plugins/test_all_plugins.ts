// Test: TypeScript type predicates ('is' keyword)
// and rest params (...args) in constructor types

// --- Test 1: Type predicates with primitives ---
function isString(val: unknown): val is string {
  return typeof val === "string";
}

function isNumber(val: unknown): val is number {
  return typeof val === "number";
}

const data: unknown[] = ["hello", 42, "world", 99];
let strings = 0;
let numbers = 0;
for (let i = 0; i < data.length; i++) {
  const v = data[i];
  if (isString(v)) strings = strings + 1;
  else if (isNumber(v)) numbers = numbers + 1;
}
console.log("Strings:", strings, "Numbers:", numbers);

// --- Test 2: Union type predicates ---
interface Cat {
  meow(): void;
}

interface Dog {
  bark(): void;
}

type Animal = Cat | Dog;

function isCat(a: Animal): a is Cat {
  return "meow" in a;
}

const myCat: Cat = { meow(): void { console.log("Meow!"); } };
if (isCat(myCat)) {
  myCat.meow();
}

// --- Test 3: Constructor type with rest params (type-only) ---
type PluginClass = new (...args: unknown[]) => object;
type Factory = new (...args: number[]) => object;

// --- Test 4: Type predicates with plugin pattern ---
interface IPlugin {
  metadata: { name: string; version: string };
  setup(): void;
  onLoad(): void;
  onEnable(): void;
  onDisable(): void;
  onUnload(): void;
}

interface PluginConst {
  metadata: { name: string; version: string };
  setup?(): void;
  onLoad?(): void;
  onEnable?(): void;
  onDisable?(): void;
  onUnload?(): void;
}

function isClassPlugin(plugin: unknown): plugin is Function {
  return typeof plugin === "function" && plugin.prototype !== undefined;
}

function isObjectPlugin(plugin: unknown): plugin is IPlugin | PluginConst {
  return typeof plugin === "object" && plugin !== null && "metadata" in (plugin as Record<string, unknown>);
}

const objectPlugin: IPlugin = {
  metadata: { name: "test", version: "1.0.0" },
  setup(): void { console.log("setup"); },
  onLoad(): void { console.log("loaded"); },
  onEnable(): void { console.log("enabled"); },
  onDisable(): void { console.log("disabled"); },
  onUnload(): void { console.log("unloaded"); },
};

console.log("isObjectPlugin:", isObjectPlugin(objectPlugin));
console.log("isClassPlugin:", isClassPlugin(objectPlugin));

// --- Test 5: Type predicate in validation ---
function validatePlugin(plugin: unknown): void {
  if (isClassPlugin(plugin)) {
    console.log("Valid class plugin");
    return;
  }
  if (isObjectPlugin(plugin)) {
    const p = plugin as Record<string, unknown>;
    const metadata = p.metadata as { name: string; version: string };
    console.log("Valid object plugin:", metadata.name);
    return;
  }
  throw new Error("Invalid plugin");
}

validatePlugin(objectPlugin);

console.log("All type predicate and rest param tests passed!");
