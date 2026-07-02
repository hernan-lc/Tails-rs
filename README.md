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

## Documentation

- [Installation](docs/installation.md) — Build instructions and feature flags
- [Usage](docs/usage.md) — CLI and library usage
- [Native Modules](docs/native-modules.md) — Module system, imports, and architecture
- [Features](docs/features.md) — Complete list of supported JavaScript/TypeScript features
- [Architecture](docs/architecture.md) — Source layout and design overview
- [Testing](docs/testing.md) — Running the test suite
- [Roadmap](docs/roadmap.md) — Completed work and planned features
