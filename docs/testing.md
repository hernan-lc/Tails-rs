## Running Tests

```bash
# Run all tests (default features)
cargo test

# Run without optional modules
cargo test --no-default-features

# Run with specific features only
cargo test --no-default-features -F fs -F path
```

## Test Targets

### Unit Tests

```bash
# Parser tests
cargo test --test unit_parser

# Lexer tests
cargo test --test unit_lexer

# VM tests
cargo test --test unit_vm

# Minified input tests
cargo test --test unit_minified
```

### Feature Integration Tests

```bash
# All features combined
cargo test --test all_features

# Async / Promises
cargo test --test async

# Classes
cargo test --test classes

# Destructuring
cargo test --test destructuring

# Error handling
cargo test --test error_handling
cargo test --test error_stack

# Functions
cargo test --test functions

# Modules (ES modules)
cargo test --test modules

# CommonJS require()
cargo test --test require_cjs

# Proxy
cargo test --test proxy

# Computed properties
cargo test --test computed_properties

# Optional chaining
cargo test --test optional_chaining

# Object accessors (getters/setters)
cargo test --test object_accessors

# Type system
cargo test --test type_system

# GC (garbage collection)
cargo test --test gc

# Buffer module
cargo test --test buffer

# Encoding (atob/btoa)
cargo test --test encoding

# FS module
cargo test --test fs_module

# Path module
cargo test --test path_module

# HTTP module
cargo test --test http_module

# Process global
cargo test --test process_global

# Intl module
cargo test --test intl
```

### Native Module Tests

```bash
# Unit tests for safe wrappers (FFI safety)
cargo test --lib ffi::

# Benchmarks
cargo bench
```

## Test Fixtures

Test fixtures live in `tests/fixtures/` and include TypeScript and CommonJS files used by integration tests. Module fixtures are in `tests/fixtures/modules/` and `tests/fixtures/require_cjs/`.
