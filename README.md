# Tails-rs

[![CI](https://github.com/nglmercer/Tails-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nglmercer/Tails-rs/actions/workflows/ci.yml)

A TypeScript-first runtime implemented in Rust.

## Overview

Tails-rs is a JavaScript/TypeScript runtime built from scratch in Rust. It compiles source code to bytecode and executes it on a stack-based virtual machine with garbage collection. It supports many modern JavaScript features including classes, promises, async/await, ES modules, and more.

## Quick Start

```bash
# Build
cargo build --release

# Run a script
tails run script.ts

# See all features in action
tails run examples/test-builtins.ts
```

## CLI

```
tails <command> [OPTIONS]

Commands:
  run <script.ts>       Run a TypeScript script (default)
  build [OPTIONS]       Build native module to dist/
  clean                 Remove dist/ directory

Run options:
  --watch               Watch for file changes and re-run automatically
  --env-file <path>     Load environment variables from a specific .env file
  --no-env-file         Disable automatic .env file loading
  --color / --no-color  Toggle colored output (default: on)
  --timestamps          Show timestamps in console output

Build options:
  --package, -p <name>  Package to build (auto-detects cdylib if omitted)
  --release             Build in release mode
  --target-dir <path>   Custom target directory
```

## Modules

| Module | Description | Import |
|--------|-------------|--------|
| `fs` | Filesystem (sync + async + streaming + watch) | `import fs from "fs"` |
| `fs/promises` | Promise-style filesystem API | `import { readFile } from "fs/promises"` |
| `path` | Path manipulation (join, resolve, parse, format, ...) | `import path from "path"` |
| `process` | Process control (exit, cwd, env, kill, ...) | `import process from "process"` |
| `os` | OS info (platform, arch, cpus, mem, ...) | `import os from "os"` |
| `http` | HTTP server | `import http from "http"` |
| `net` | TCP client (createConnection) | `import net from "net"` |
| `url` | URL parsing + URLSearchParams | `import { URL } from "url"` |
| `events` | EventEmitter | `import { EventEmitter } from "events"` |
| `crypto` | Random bytes/UUID, hashing | `import crypto from "crypto"` |
| `buffer` | Binary data (also available as global `Buffer`) | `import { Buffer } from "buffer"` |
| `child_process` | exec, execSync, spawn | `import cp from "child_process"` |
| `assert` | Assertions (strictEqual, deepEqual, ...) | `import assert from "assert"` |
| `intl` | Intl.DateTimeFormat, Intl.NumberFormat | Available as global `Intl` |

Global objects (no import needed): `console`, `Math`, `JSON`, `Promise`, `Map`, `Set`, `Date`, `RegExp`, `URL`, `fetch`, `Headers`, `Request`, `Response`, `Buffer`, `process`, `Proxy`, `Reflect`, `Symbol`, `Error` family, `TypedArray` family, `WeakMap`, `WeakSet`, `Generator`, `WebSocket`.

## Documentation
