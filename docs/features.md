## Variables & Types
- **Declaration**: `let`, `const`, `var`
- **Primitives**: `number`, `string`, `boolean`, `undefined`, `null`, `bigint`
- **Operators**: Arithmetic (`+`, `-`, `*`, `/`, `%`, `**`), compound assignment (`+=`, `-=`, etc.), comparison (`==`, `===`, `!=`, `!==`, `<`, `>`, `<=`, `>=`), logical (`&&`, `||`, `!`), bitwise (`~`)
- **Increment/Decrement**: `++`, `--`
- **Type Inspection**: `typeof`, `void`

## Control Flow
- **Conditionals**: `if` / `else if` / `else`, ternary `? :`
- **Loops**: `for`, `while`, `do...while`, `for...in`
- **Jump**: `break`, `continue`, `switch` / `case` / `default`

## Functions
- Declarations and expressions
- Arrow functions (with and without braces)
- Closures and lexical scoping
- Higher-order functions
- `Function.prototype.bind()`, `.call()`, `.apply()`

## Classes (OOP)
- Class declarations and expressions
- Constructors and instance methods
- Static methods
- Getters and setters
- Inheritance with `extends` and `super`
- `instanceof` operator

## Objects & Arrays

**Object methods**
- `Object.keys()`, `Object.values()`, `Object.entries()`
- `Object.assign()`, `Object.defineOwnProperty()`
- `Object.is()`, `Object.freeze()`, `Object.seal()`, `Object.isExtensible()`, `Object.preventExtensions()`, `Object.isFrozen()`, `Object.isSealed()`
- `Object.getOwnPropertyDescriptor()`
- `Object.hasOwnProperty()`

**Array methods**
- Mutation: `push()`, `pop()`, `shift()`, `unshift()`, `splice()`
- Iteration: `map()`, `filter()`, `reduce()`, `forEach()`, `find()`, `findIndex()`
- Inspection: `some()`, `every()`, `indexOf()`, `includes()`
- Transformation: `join()`, `reverse()`, `sort()`, `concat()`, `slice()`, `flat()`
- Additional: `copyWithin()`, `fill()`, `findLast()`, `findLastIndex()`, `flatMap()`, `lastIndexOf()`
- Static: `Array.isArray()`, `Array.from()`, `Array.of()`

**Typed Arrays**
- Constructors: `Int8Array`, `Uint8Array`, `Uint8ClampedArray`, `Int16Array`, `Uint16Array`, `Int32Array`, `Uint32Array`, `Float32Array`, `Float64Array`, `BigInt64Array`, `BigUint64Array`
- Static methods: `from()`, `of()`
- Instance methods: `get()`, `set()`, `subarray()`, `slice()`
- Properties: `length`, `byteLength`, `byteOffset`, `BYTES_PER_ELEMENT`

**ES6+ Collections**
- **Map**: `new Map()`, `get()`, `set()`, `has()`, `delete()`, `clear()`, `forEach()`, `keys()`, `values()`, `entries()`, `size`
- **Set**: `new Set()`, `add()`, `has()`, `delete()`, `clear()`, `forEach()`, `keys()`, `values()`, `entries()`, `size`
- **WeakMap**: `new WeakMap()`, `get()`, `set()`, `has()`, `delete()`
- **WeakSet**: `new WeakSet()`, `add()`, `has()`, `delete()`

## Strings
- `charAt()`, `charCodeAt()`
- `slice()`, `substring()`
- `indexOf()`, `includes()`
- `replace()`, `split()`, `trim()`
- Case conversion: `toLowerCase()`, `toUpperCase()`
- Testing: `startsWith()`, `endsWith()`
- Padding: `padStart()`, `padEnd()`, `repeat()`
- `matchAll()` with RegExp support

## Math
- Constants: `Math.PI`, `Math.E`
- Functions: `abs()`, `floor()`, `ceil()`, `round()`, `min()`, `max()`, `pow()`, `sqrt()`, `log()`, `sin()`, `cos()`, `tan()`
- `Math.random()`

## JSON
- `JSON.stringify()`
- `JSON.parse()`

## Promise & Async
- `Promise` constructor, `resolve`, `reject`
- `.then()`, `.catch()`, `.finally()`
- `Promise.all()`, `Promise.race()`, `Promise.allSettled()`, `Promise.any()`, `Promise.withResolvers()`
- `await` operator
- `for await...of` async iteration
- Timers: `setTimeout()`, `setInterval()`, `clearInterval()`

## Error Handling
- `try` / `catch` / `finally`
- `throw` with any value
- Error subclasses with real stack traces (`Error`, `TypeError`, `ReferenceError`, `SyntaxError`, `RangeError`)

## Global Functions
- `parseInt()`, `parseFloat()`
- `isNaN()`, `isFinite()`
- `Number.parseInt()`, `Number.parseFloat()`
- `Number.isNaN()`, `Number.isFinite()`
- `Number.isInteger()`, `Number.isSafeInteger()`

## Encoding
- `atob()` / `btoa()` — Base64 encoding/decoding

## Buffer (native module)
- `Buffer.alloc()`, `Buffer.from()`, `Buffer.concat()`, `Buffer.isBuffer()`, `Buffer.byteLength()`
- Instance: `toString()`, `write()`, `slice()`, `copy()`, `fill()`, `compare()`, `equals()`, `indexOf()`

## process (native module)
- `process.platform`, `process.arch`, `process.pid`
- `process.cwd()`, `process.chdir()`
- `process.env`, `process.argv`
- `process.exit()`, `process.stdout.write()`, `process.stderr.write()`
- `process.hrtime()`, `process.hrtime.bigint()`, `process.nextTick()`

## Intl (native module)
- `Intl.DateTimeFormat` — Date/time formatting with `format()` and `formatToParts()`
- `Intl.NumberFormat` — Number formatting with decimal, currency, and percent styles

## fs (native module)
- Sync: `readFileSync()`, `writeFileSync()`, `existsSync()`, `mkdirSync()`, `readdirSync()`, `statSync()`, `unlinkSync()`, `rmSync()`, `copyFileSync()`, `renameSync()`, `appendFileSync()`
- Async: `readdir()`, `readFile()`, `writeFile()`, `stat()`, `mkdir()`, `unlink()`, `copyFile()`, `rename()`

## path (native module)
- `path.join()`, `path.resolve()`, `path.basename()`, `path.dirname()`, `path.extname()`, `path.relative()`, `path.isAbsolute()`, `path.normalize()`
- `path.sep`, `path.delimiter`

## os (native module)
- `os.platform()`, `os.arch()`, `os.cpus()`, `os.totalmem()`, `os.freemem()`, `os.uptime()`, `os.hostname()`, `os.type()`, `os.release()`, `os.homedir()`, `os.tmpdir()`

## http (native module)
- `http.createServer()`, `server.listen()`, `server.close()`
- Request: `req.on('data'/'end')`, `req.body`, `req.headers`, `req.method`, `req.url`
- Response: `res.writeHead()`, `res.write()`, `res.end()`
- Options: `maxConnections`, `timeoutMs`

## websocket (native module)
- `new WebSocket(url)`, `ws.send()`, `ws.close()`
- `ws.addEventListener('open'/'message'/'error'/'close', callback)`
- `ws.removeEventListener()`

## fetch (built-in)
- `fetch(url, options)` — HTTP client with Promise-based API
- `Headers` — `append()`, `get()`, `set()`, `has()`, `delete()`, `forEach()`, `keys()`, `values()`, `entries()`
- `Request` — constructor with method, headers, body
- `Response` — `text()`, `json()`, `arrayBuffer()`, static methods (`json()`, `error()`, `redirect()`, `clone()`)

## URL (built-in)
- `new URL(url, base)` — URL parsing and manipulation
- `URLSearchParams` — query string handling (`get()`, `getAll()`, `has()`, `set()`, `append()`, `delete()`, `toString()`, `entries()`, `keys()`, `values()`, `forEach()`)
- `URL.canParse()`, `URL.parse()`
- `url.toJSON()`
- `url.pathname`, `url.search`, `url.hash`, etc.
- `fileURLToPath()` — convert `file://` URLs to filesystem paths

## assert (built-in)
- `assert(value, message)` — throws if value is falsy
- `assertStrictEqual(actual, expected, message)` — throws if not strictly equal

## child_process (built-in)
- `execSync(command)` — execute command synchronously, returns stdout
- `exec(command, callback)` — execute command asynchronously
- `spawn(command, args, options)` — spawn a child process

## Destructuring & Spread
- Array destructuring with skipping
- Object destructuring with aliasing
- Array spread operator (`...`)

## Iterators & Generators
- `function*` generators with `yield`
- `.next()`, `.return()`, `.throw()` on generator instances
- `for...of` loop with `Symbol.iterator`
- `for await...of` with `Symbol.asyncIterator`
- **Iterator helpers**: `map()`, `filter()`, `take()`, `drop()`, `forEach()`, `toArray()` with chaining
- Generator `[Symbol.iterator]` support

## Proxy & Reflect
- **Proxy** objects with handlers (get, set, has, deleteProperty, apply, construct)
- **Reflect** API — declared but not yet implemented (all methods are stubs returning defaults)

## Symbol
- `Symbol()`, `Symbol.for()`, `Symbol.keyFor()`
- Well-known symbols: `Symbol.iterator`, `Symbol.toStringTag`, `Symbol.hasInstance`, `Symbol.asyncIterator`

## Other
- **Type annotations** (TypeScript) — parsed and ignored at runtime
- **Optional chaining** (`?.`) and **nullish coalescing** (`??`)
- **CommonJS**: `require()` with `module.exports`/`exports`, `__dirname`/`__filename`, module caching, circular dependencies
- **ES Modules**: `import` / `export` (named, default, namespace)
- **Dotenv**: Auto-loading `.env` files
- **BigInt**: Full primitive type with `42n` literals, arithmetic, comparison, `BigInt()` constructor
- **Date**: `new Date()`, getters/setters, ISO parsing, `Date.now()`, `Date.parse()`, `Date.UTC()`
- **RegExp**: `new RegExp()`, `test()`, `exec()`, flags (`g`, `i`, `m`, `s`, `u`, `y`), `String.prototype.match/replace/search/matchAll`
- **console**: `log()`, `warn()`, `error()`, `info()`, `table()`, `dir()`, `group()`, `groupEnd()`, `groupCollapsed()`, `time()`, `timeEnd()`, `assert()`, `clear()`
