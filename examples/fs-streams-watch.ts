// fs.createReadStream / fs.watch usage example for Tails-rs
// Run: tails run examples/fs-streams-watch.ts
//
// Demonstrates: streaming reads of large files in fixed-size chunks,
// and polling-based directory change detection.

import fs from "../dist/tails-fs.native";
import os from "../dist/tails-os.native";

const tmpFile = os.tmpdir() + "/tails-streams-" + os.pid() + ".bin";
const tmpDir = os.tmpdir() + "/tails-watch-" + os.pid();

// ---------------------------------------------------------------------------
// createReadStream
// ---------------------------------------------------------------------------

console.log("=== fs.createReadStream ===");

// Write a 32-byte payload to read back in 8-byte chunks.
const payload = "0123456789ABCDEFGHIJKLMNOPQRSTUV";
fs.write_file(tmpFile, payload);

const open = JSON.parse(fs.create_read_stream(tmpFile));
console.log("open:", open);

let collected = "";
let chunkIdx = 0;
while (true) {
    const chunk = JSON.parse(fs.stream_read(open.id, 8));
    if (chunk.done) break;
    // chunk.data is base64-encoded; Buffer.from handles decoding.
    const text = Buffer.from(chunk.data, "base64").toString("utf8");
    collected += text;
    console.log(`chunk ${chunkIdx++}: ${text}`);
    if (chunkIdx > 10) break; // safety
}

console.log("collected:", collected);
console.log("matches payload:", collected === payload);
fs.stream_close(open.id);
fs.unlink(tmpFile);

// ---------------------------------------------------------------------------
// watch
// ---------------------------------------------------------------------------

console.log("\n=== fs.watch ===");

fs.mkdir(tmpDir, true);
const w = JSON.parse(fs.watch(tmpDir, 50));
console.log("watcher:", w);

// Mutate the directory and poll. In the Tails runtime the
// watcher's `poll()` re-snapshots on every call when the interval
// has elapsed, so the very first poll after the interval will see
// the new file.
fs.write_file(tmpDir + "/created.txt", "hi");
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let seen = [];
for (let i = 0; i < 30 && seen.length === 0; i++) {
    await sleep(20);
    seen = JSON.parse(fs.watch_poll(w.id));
}
console.log("events after create:", seen);

// Mutate again — append to the file — and poll.
fs.write_file(tmpDir + "/created.txt", "hi again");
seen = [];
for (let i = 0; i < 30 && seen.length === 0; i++) {
    await sleep(20);
    seen = JSON.parse(fs.watch_poll(w.id));
}
console.log("events after modify:", seen);

// Remove the file and poll.
fs.unlink(tmpDir + "/created.txt");
seen = [];
for (let i = 0; i < 30 && seen.length === 0; i++) {
    await sleep(20);
    seen = JSON.parse(fs.watch_poll(w.id));
}
console.log("events after delete:", seen);

fs.watch_close(w.id);
fs.unlink(tmpDir);
console.log("\n=== Done ===");
