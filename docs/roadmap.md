> Based on current implementation status. Contributions welcome!

### Recently Completed
- **Modular Native Modules** — `fs`, `path`, `process`, `os` extracted to standalone crates under `modules/` with Cargo feature flags for selective inclusion
- **Bare-name Imports** — `import fs from "fs"` works alongside the legacy `import fs from "./fs.native"` syntax
- **Light Runtime** — Moved process, Buffer, Intl from globals to import-only native modules
- **Native Module System** — Import-only modules via `.native` extension, registry-based loader
- **events module** — EventEmitter class with `on()`, `emit()`, `off()`, `listenerCount()`
- **os module** — OS info: `platform()`, `arch()`, `cpus()`, `totalmem()`, `freemem()`, `uptime()`, `hostname()`, `type()`, `release()`, `homedir()`, `tmpdir()`
- **crypto module** — `randomBytes()`, `randomUUID()`, `createHash()` (SHA-224/256/384/512)
- **process module** — `process.platform`, `process.arch`, `process.pid`, `process.cwd()`, `process.chdir()`, `process.env`, `process.argv`, `process.exit()`, `process.stdout.write()`, `process.hrtime()`, `process.nextTick()`
- **buffer module** — `Buffer.alloc()`, `Buffer.from()`, `Buffer.concat()`, `Buffer.isBuffer()`, `Buffer.byteLength()`, `toString()`, `write()`, `slice()`, `copy()`, `fill()`, `compare()`, `equals()`, `indexOf()`
- **intl module** — `Intl.DateTimeFormat` with `format()` and `formatToParts()`, `Intl.NumberFormat` with currency and percent styles
- **Import-only fs/path** — `fs` and `path` removed from globals, now require explicit import
- **Reflect API** — Native implementations for `get`, `set`, `apply`, `construct`, `isExtensible`, `preventExtensions`, etc.
- **Generators** — Runtime support for `function*`, `yield`, and `.next()`
- **for...of loop** — Iterator protocol execution with `Symbol.iterator`
- **Symbol** type and well-known symbols (`Symbol.iterator`, `Symbol.toStringTag`, `Symbol.hasInstance`, `Symbol.asyncIterator`, etc.)
- **Function prototypes** — `Function.prototype.bind()`, `.call()`, `.apply()`
- **Array enhancements** — `copyWithin`, `fill`, `findLast`, `findLastIndex`, `flatMap`, `lastIndexOf`, `Array.isArray()`, `Array.from()`, `Array.of()`
- **Object methods** — `Object.is()`, `Object.seal()`, `Object.isSealed()`, `Object.freeze()`, `Object.isFrozen()`, `Object.isExtensible()`, `Object.preventExtensions()`
- **Promise enhancements** — `Promise.any()`, `Promise.allSettled()`, `Promise.withResolvers()`
- **BigInt** — Full primitive type with literals (`42n`), arithmetic, comparison, `BigInt()` constructor
- **Date** — Full implementation with `new Date()`, getters/setters, ISO string parsing, `Date.now()`, `Date.parse()`, `Date.UTC()`
- **RegExp** — Full implementation with `new RegExp()`, `test()`, `exec()`, flags support, `String.prototype.match/replace/search`
- **Iterator helpers** — `map()`, `filter()`, `take()`, `drop()`, `forEach()`, `toArray()` on array/string iterators, with chaining support
- **for await...of** — Async iteration with `Symbol.asyncIterator` support, automatic promise resolution
- **Error stack traces** — Real stack traces with function names for `Error`, `TypeError`, `ReferenceError`, `SyntaxError`, `RangeError`
- **Encoding** — `atob()` and `btoa()` for base64 encoding/decoding
- **path module** — `path.join()`, `path.resolve()`, `path.basename()`, `path.dirname()`, `path.extname()`, `path.relative()`, `path.isAbsolute()`, `path.normalize()`, `path.sep`, `path.delimiter`
- **fs module** — `fs.readFileSync()`, `fs.writeFileSync()`, `fs.existsSync()`, `fs.mkdirSync()`, `fs.readdirSync()`, `fs.statSync()`, `fs.unlinkSync()`, `fs.rmSync()`, `fs.copyFileSync()`, `fs.renameSync()`, `fs.appendFileSync()`
- **CommonJS Support** — `require()` for CJS module loading with `module.exports`/`exports`, `__dirname`/`__filename` globals, module caching, circular dependency handling, `.cjs` extension support, native module resolution via `require("path")` etc.

### Future / Research
- **More Native Modules**
  - `url` — URL parsing and manipulation
  - `stream` — Stream processing
  - `http` / `https` — HTTP client/server
  - `child_process` — Process spawning
- **Web APIs & FFI**
  - `fetch` and `Response`/`Request` types
  - Enhanced FFI for Rust interop
- **Performance**
  - JIT compilation tier
  - Optimized GC (generational, concurrent)
- **Node.js Compatibility**
  - Additional core modules (url, crypto, stream, etc.)
