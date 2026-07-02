Native modules are imported by bare name. They are **not** available as globals — you must always import them explicitly.

```typescript
import fs from "fs";
import path from "path";
import process from "process";
import os from "os";
import Buffer from "./buffer.native";
import Intl from "./intl.native";
import events from "./events.native";
import crypto from "./crypto.native";
```

The `.native` extension still works for all modules:

```typescript
import fs from "./fs.native";
import path from "./path.native";
```

## Available Native Modules

| Module | Feature | Crate | Description |
|--------|---------|-------|-------------|
| `fs` | `fs` | `modules/fs` | File system operations (read, write, stat, mkdir, etc.) |
| `path` | `path` | `modules/path` | Path manipulation (join, resolve, basename, etc.) |
| `process` | `process` | `modules/process` | Process info and control (env, argv, exit, etc.) |
| `os` | `os` | `modules/os` | OS information (platform, arch, cpus, memory, etc.) |
| `http` | `http` | `modules/http` | HTTP/1.1 server (`createServer`, `listen`, `req`/`res` objects) |
| `buffer` | *(always)* | *(built-in)* | Node.js-compatible binary data handling |
| `intl` | *(always)* | *(built-in)* | Internationalization (DateTimeFormat, NumberFormat) |
| `events` | *(always)* | *(built-in)* | EventEmitter class with on/emit/off |
| `crypto` | *(always)* | *(built-in)* | Cryptographic functions (randomBytes, randomUUID, createHash) |

## Module Architecture

Each feature-gated module is split into two layers:

- **`modules/<name>/`** — Pure Rust implementation with no dependency on the runtime. Contains the actual business logic (fs operations, path manipulation, etc.)
- **`src/runtime_env/native_fns/<name>_fns.rs`** — Thin adapter that converts between runtime `Value` types and the pure module functions

This separation keeps the core runtime lightweight and the module logic testable independently.

## Creating Native Modules with Proc Macros

Custom native modules can be built as separate crates using the `tails-native-macros` crate. This eliminates manual FFI boilerplate.

### Setup

Add dependencies to your `Cargo.toml`:

```toml
[package]
name = "my-module"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
tails-abi = { path = "../abi" }
tails-native-macros = { path = "../native-macros" }
```

### Simple Functions

Use `#[tails_function]` to export individual functions:

```rust
use tails_native_macros::{tails_function, tails_module};

#[tails_function]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[tails_function]
pub fn add(a: f64, b: f64) -> f64 {
    a + b
}

#[tails_module(name = "my-module")]
mod my_module {
    use super::*;
    // Functions above are automatically registered
}
```

The macro generates:
- FFI wrapper with the correct `extern "C"` signature
- Type conversion between NativeValue and Rust types
- DTS metadata for TypeScript definitions
- Error handling

### Class-Based Modules

Use `#[tails_class]` on an `impl` block for struct-based classes:

```rust
use tails_native_macros::{tails_class, tails_function, tails_module};

pub struct Counter {
    count: f64,
}

#[tails_class]
impl Counter {
    pub fn new(initial: f64) -> Self {
        Counter { count: initial }
    }

    pub fn increment(&mut self) {
        self.count += 1.0;
    }

    pub fn decrement(&mut self) {
        self.count -= 1.0;
    }

    pub fn get_count(&self) -> f64 {
        self.count
    }
}

#[tails_module(name = "my-module")]
mod my_module {
    use super::*;
}
```

The macro generates:
- Instance registry with atomic ID allocation
- Constructor FFI wrapper (methods returning `Self`)
- Method FFI wrappers with instance lookup
- Registration with camelCase naming (`counter_getCount`)

### Module Declaration

The `#[tails_module]` attribute goes on a `mod` block. It auto-registers all `#[tails_function]` items and class methods:

```rust
#[tails_module(name = "my-module")]
mod my_module {
    use super::*;
    use tails_native_macros::{tails_class, tails_function};

    #[tails_function]
    pub fn helper() -> String { "ok".into() }

    pub struct MyClass { /* ... */ }

    #[tails_class]
    impl MyClass {
        pub fn create() -> Self { /* ... */ }
    }
}
```

### Building and Using

```bash
# Build the native module
tails build -p my-module

# Run a script that imports it
tails run examples/my-example.ts
```

The build produces `dist/lib<name>.so` (or `.dylib`/`.dll`) and `dist/<name>.d.ts`.

### Type Mapping

Rust types are automatically converted to TypeScript:

| Rust | TypeScript |
|------|------------|
| `f64`, `f32`, `i64`, `i32`, etc. | `number` |
| `String`, `&str` | `string` |
| `bool` | `boolean` |
| `Vec<T>` | `Array<T>` |
| `Option<T>` | `T \| null` |
| `HashMap<K, V>` | `Record<K, V>` |
| `()` | `void` |
