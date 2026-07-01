// Comprehensive test for native modules: fs, path, os
// Run: cargo run --bin tails -- run examples/test-native-modules.ts

import fs from "../dist/tails-fs.native";
import path from "../dist/tails-path.native";
import os from "../dist/tails-os.native";

let total = 0;
let passed = 0;
let failed = 0;

function test(name: string, fn: () => void) {
  total++;
  try {
    fn();
    passed++;
    console.log(`  [PASS] ${name}`);
  } catch (e) {
    failed++;
    console.log(`  [FAIL] ${name}: ${e}`);
  }
}

function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

function assertEqual(a: unknown, b: unknown, msg: string) {
  if (a !== b) throw new Error(`${msg}: expected ${JSON.stringify(b)}, got ${JSON.stringify(a)}`);
}

// ============================================================================
// FS Module Tests
// ============================================================================

console.log("\n=== FS Module ===");

const testDir = os.tmpdir() + "/tails-test-" + os.pid();
const testFile = testDir + "/test.txt";
const testFile2 = testDir + "/test2.txt";
const testFile3 = testDir + "/copy.txt";

test("mkdir recursive", () => {
  assert(fs.mkdir(testDir, true), "mkdir should succeed");
});

test("exists on created dir", () => {
  assert(fs.exists(testDir), "directory should exist");
});

test("write_file", () => {
  assert(fs.write_file(testFile, "Hello, Tails!"), "write should succeed");
});

test("read_file", () => {
  const content = fs.read_file(testFile);
  assertEqual(content, "Hello, Tails!", "content should match");
});

test("exists on created file", () => {
  assert(fs.exists(testFile), "file should exist");
});

test("stat on file", () => {
  const raw = fs.stat(testFile);
  const stat = JSON.parse(raw);
  assert(stat.isFile === true, "should be a file");
  assert(stat.size > 0, "size should be positive");
});

test("append_file", () => {
  assert(fs.append_file(testFile, "\nAppended line"), "append should succeed");
  const content = fs.read_file(testFile);
  assert(content.includes("Appended line"), "should contain appended text");
});

test("write_file overwrite", () => {
  assert(fs.write_file(testFile, "Overwritten"), "overwrite should succeed");
  assertEqual(fs.read_file(testFile), "Overwritten", "should have new content");
});

test("copy_file", () => {
  assert(fs.copy_file(testFile, testFile3) > 0, "copy should succeed");
  assertEqual(fs.read_file(testFile3), "Overwritten", "copy should match source");
});

test("rename", () => {
  assert(fs.rename(testFile, testFile2), "rename should succeed");
  assert(!fs.exists(testFile), "old path should not exist");
  assert(fs.exists(testFile2), "new path should exist");
  assertEqual(fs.read_file(testFile2), "Overwritten", "content should be preserved");
});

test("read_file_bytes", () => {
  const raw = fs.read_file_bytes(testFile2);
  const encoded = JSON.parse(raw);
  assert(typeof encoded === "string", "should return a string");
});

test("write_file_bytes", () => {
  const data = "VGVzdA=="; // base64 of "Test"
  assert(fs.write_file_bytes(testFile, data), "write bytes should succeed");
  const content = fs.read_file(testFile);
  assertEqual(content, "Test", "decoded content should match");
});

test("readdir", () => {
  const raw = fs.readdir(testDir);
  const entries = JSON.parse(raw);
  assert(Array.isArray(entries), "should return array");
  assert(entries.length >= 2, "should have at least 2 entries");
});

test("lstat", () => {
  const raw = fs.lstat(testFile2);
  const stat = JSON.parse(raw);
  assert(stat.isFile === true, "should be a file");
});

test("readlink (non-symlink)", () => {
  const target = fs.readlink(testFile2);
  assertEqual(target, "", "non-symlink should return empty");
});

test("realpath", () => {
  const real = fs.realpath(testFile2);
  assert(typeof real === "string" && real.length > 0, "should return absolute path");
});

test("unlink", () => {
  assert(fs.unlink(testFile2), "unlink should succeed");
  assert(!fs.exists(testFile2), "file should not exist after unlink");
});

test("rm file", () => {
  assert(fs.write_file(testFile, "to delete"), "write should succeed");
  assert(fs.rm(testFile, false), "rm should succeed");
  assert(!fs.exists(testFile), "file should not exist after rm");
});

test("chmod (unix)", () => {
  assert(fs.write_file(testFile, "chmod test"), "write should succeed");
  assert(fs.chmod(testFile, 420), "chmod should succeed"); // 420 = 0o644
  fs.rm(testFile, false);
});

test("cleanup test dir", () => {
  assert(fs.rm(testDir, true), "recursive rm should succeed");
  assert(!fs.exists(testDir), "dir should not exist after rm");
});

// ============================================================================
// PATH Module Tests
// ============================================================================

console.log("\n=== PATH Module ===");

test("sep", () => {
  const s = path.sep();
  assert(s === "/" || s === "\\", "sep should be / or \\");
});

test("delimiter", () => {
  const d = path.delimiter();
  assert(d === ":" || d === ";", "delimiter should be : or ;");
});

test("join", () => {
  const parts = JSON.stringify(["usr", "local", "bin"]);
  const result = path.join(parts);
  assert(result.includes("usr"), "should contain usr");
  assert(result.includes("bin"), "should contain bin");
});

test("resolve", () => {
  const parts = JSON.stringify(["/tmp", "test"]);
  const result = path.resolve(parts);
  assert(result.startsWith("/"), "should be absolute");
  assert(result.includes("tmp"), "should contain tmp");
});

test("basename", () => {
  assertEqual(path.basename("/home/user/file.txt", ""), "file.txt", "basename without ext");
  assertEqual(path.basename("/home/user/file.txt", ".txt"), "file", "basename with ext");
});

test("dirname", () => {
  assertEqual(path.dirname("/home/user/file.txt"), "/home/user", "dirname");
  assertEqual(path.dirname("file.txt"), ".", "dirname of relative");
});

test("extname", () => {
  assertEqual(path.extname("file.txt"), ".txt", "extname");
  assertEqual(path.extname("file"), "", "no ext");
  assertEqual(path.extname(".hidden"), "", "dotfile");
});

test("filename", () => {
  assertEqual(path.filename("/home/user/file.txt"), "file", "filename");
  assertEqual(path.filename("archive.tar.gz"), "archive.tar", "filename with multiple dots");
});

test("is_absolute", () => {
  assert(path.is_absolute("/home"), "should be absolute");
  assert(!path.is_absolute("home"), "should not be absolute");
});

test("normalize", () => {
  assertEqual(path.normalize("/home/user/../file"), "/home/file", "normalize parent");
  assertEqual(path.normalize("/home/./file"), "/home/file", "normalize current");
});

test("relative", () => {
  const result = path.relative("/home/user", "/home/user/docs/file.txt");
  assert(result.includes("file.txt"), "should contain file.txt");
});

test("parse", () => {
  const raw = path.parse("/home/user/file.txt");
  const parsed = JSON.parse(raw);
  assertEqual(parsed.ext, ".txt", "parsed ext");
  assertEqual(parsed.name, "file", "parsed name");
});

test("format", () => {
  const parts = JSON.stringify({ dir: "/home/user", base: "file.txt" });
  const result = path.format(parts);
  assert(result.includes("file.txt"), "should contain file.txt");
});

test("contains", () => {
  assert(path.contains("/home/user/docs", "docs"), "should contain path segment");
  assert(!path.contains("/home/user/docs", "other"), "should not contain other");
});

// ============================================================================
// OS Module Tests
// ============================================================================

console.log("\n=== OS Module ===");

test("platform", () => {
  const p = os.platform();
  assert(["linux", "darwin", "win32", "unknown"].includes(p), `platform should be valid: ${p}`);
});

test("arch", () => {
  const a = os.arch();
  assert(["x64", "arm64", "x86", "unknown"].includes(a), `arch should be valid: ${a}`);
});

test("type_name", () => {
  assertEqual(os.type_name(), "Tails", "type_name");
});

test("endianness", () => {
  const e = os.endianness();
  assert(e === "LE" || e === "BE", "endianness should be LE or BE");
});

test("totalmem", () => {
  const mem = os.totalmem();
  assert(mem > 0, "totalmem should be positive");
});

test("freemem", () => {
  const mem = os.freemem();
  assert(mem > 0, "freemem should be positive");
});

test("uptime", () => {
  const u = os.uptime();
  assert(u > 0, "uptime should be positive");
});

test("hostname", () => {
  const h = os.hostname();
  assert(typeof h === "string" && h.length > 0, "hostname should be non-empty string");
});

test("os_type", () => {
  const t = os.os_type();
  assert(typeof t === "string" && t.length > 0, "os_type should be non-empty");
});

test("release", () => {
  const r = os.release();
  assert(typeof r === "string" && r.length > 0, "release should be non-empty");
});

test("homedir", () => {
  const h = os.homedir();
  assert(typeof h === "string" && h.length > 0, "homedir should be non-empty");
});

test("tmpdir", () => {
  const t = os.tmpdir();
  assert(typeof t === "string" && t.length > 0, "tmpdir should be non-empty");
});

test("pid", () => {
  const p = os.pid();
  assert(p > 0, "pid should be positive");
});

test("env_var", () => {
  const home = os.env_var("HOME");
  assert(typeof home === "string", "HOME env var should be a string");
});

test("env_vars", () => {
  const raw = os.env_vars();
  const vars = JSON.parse(raw);
  assert(Array.isArray(vars), "should return array");
  assert(vars.length > 0, "should have at least one env var");
});

test("cpus", () => {
  const raw = os.cpus();
  const cpus = JSON.parse(raw);
  assert(Array.isArray(cpus), "should return array");
  assert(cpus.length > 0, "should have at least one CPU");
});

test("loadavg", () => {
  const raw = os.loadavg();
  const avg = JSON.parse(raw);
  assert(Array.isArray(avg) && avg.length === 3, "should return array of 3");
  assert(avg[0] >= 0, "load avg should be non-negative");
});

test("getuid (unix)", () => {
  const uid = os.getuid();
  assert(typeof uid === "number", "uid should be a number");
});

test("getgid (unix)", () => {
  const gid = os.getgid();
  assert(typeof gid === "number", "gid should be a number");
});

test("geteuid (unix)", () => {
  const euid = os.geteuid();
  assert(typeof euid === "number", "euid should be a number");
});

test("getegid (unix)", () => {
  const egid = os.getegid();
  assert(typeof egid === "number", "egid should be a number");
});

// ============================================================================
// Summary
// ============================================================================

console.log(`\n${"=".repeat(50)}`);
console.log(`Total: ${total} tests, Passed: ${passed}, Failed: ${failed}`);
console.log(`${"=".repeat(50)}`);

if (failed > 0) {
  console.log("\nSome tests failed!");
  process.exit(1);
} else {
  console.log("\nAll tests passed!");
}
