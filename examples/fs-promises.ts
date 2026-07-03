// fs/promises usage example for Tails-rs
// Run: tails run examples/fs-promises.ts
//
// Demonstrates: import fs from "fs/promises" and the awaitable
// Promise-style API. Every function returns a JSON envelope
// `{ok: true, value: <x>}` on success or `{ok: false, error: "..."}`
// on failure.

import fs from "fs/promises";
import os from "../dist/tails-os.native";

console.log("=== fs/promises ===");

const tmpDir = os.tmpdir() + "/tails-fs-promises-" + os.pid();
const file1 = tmpDir + "/hello.txt";
const file2 = tmpDir + "/copy.txt";

// ---------------------------------------------------------------------------
// readFile / writeFile
// ---------------------------------------------------------------------------

console.log("\n--- readFile / writeFile ---");

const w = JSON.parse(await fs.write_file(file1, "Hello, async fs!"));
console.log("write_file:", w);
const r = JSON.parse(await fs.read_file(file1));
console.log("read_file value:", r.value);

const missing = JSON.parse(await fs.read_file("/tmp/__does_not_exist__.txt"));
console.log("missing file error envelope:", missing);

// ---------------------------------------------------------------------------
// mkdir / readdir
// ---------------------------------------------------------------------------

console.log("\n--- mkdir / readdir ---");

const m = JSON.parse(await fs.mkdir(tmpDir, true));
console.log("mkdir:", m);
await fs.write_file(tmpDir + "/a.txt", "a");
await fs.write_file(tmpDir + "/b.txt", "b");
const dir = JSON.parse(await fs.readdir(tmpDir));
console.log("readdir value:", dir.value);

await fs.unlink(tmpDir + "/a.txt");
await fs.unlink(tmpDir + "/b.txt");

// ---------------------------------------------------------------------------
// stat / exists / unlink
// ---------------------------------------------------------------------------

console.log("\n--- stat / exists / unlink ---");

const s = JSON.parse(await fs.stat(file1));
console.log("stat value:", s.value);

const ex1 = JSON.parse(await fs.exists(file1));
console.log("exists (yes):", ex1.value);

await fs.unlink(file1);
const ex2 = JSON.parse(await fs.exists(file1));
console.log("exists (after unlink):", ex2.value);

// ---------------------------------------------------------------------------
// copy / rename
// ---------------------------------------------------------------------------

console.log("\n--- copy / rename ---");

await fs.write_file(file1, "copy me");
const c = JSON.parse(await fs.copy_file(file1, file2));
console.log("copy_file value:", c.value);

const r1 = JSON.parse(await fs.read_file(file2));
console.log("read after copy:", r1.value);

const mv = JSON.parse(await fs.rename(file1, file1 + ".renamed"));
console.log("rename:", mv);

// Cleanup
const renamed = JSON.parse(await fs.read_file(file1 + ".renamed"));
console.log("read after rename:", renamed.value);
await fs.unlink(file1 + ".renamed");
await fs.unlink(file2);
await fs.unlink(tmpDir);

console.log("\n=== Done ===");
