## Source Layout

```
src/
├── main.rs             # CLI binary entry point
├── lib.rs              # Library crate root
├── runtime.rs          # RuntimeConfig, TailsRuntime implementation
├── dotenv.rs           # .env file loading
├── compiler/           # Lexer, parser, bytecode generator, type checker
│   ├── lexer.rs        # Tokenizer
│   ├── parser/         # Expressions, statements, types
│   ├── type_checker/   # Type checking with narrowing
│   └── bytecode/       # AST to bytecode compilation
├── vm/                 # Virtual machine
│   └── interpreter/    # Stack-based bytecode interpreter
│       ├── instructions.rs
│       ├── builtins.rs        # Global object registration
│       ├── native_loader.rs   # Registry-based native module loader
│       ├── calls.rs           # Function call dispatch
│       ├── modules.rs         # Module loading
│       ├── promise_runtime.rs # Promise/async implementation
│       ├── exception_handling.rs
│       ├── iterators.rs       # Iterator protocol
│       ├── class_ops.rs       # Class/inheritance
│       ├── property_access.rs # Property get/set dispatch
│       ├── safe_function.rs   # Safe function pointer wrappers
│       ├── safe_library.rs    # Safe library loading wrappers
│       └── ...
├── runtime_env/        # Native function adapters and async runtime
│   ├── builtins.rs     # Built-in object setup
│   ├── modules.rs      # Module registry
│   ├── async_runtime.rs
│   ├── weak_refs.rs
│   └── native_fns/     # 40+ native function modules
│       ├── constants.rs       # NATIVE_TABLE index constants
│       ├── console.rs, math_fns.rs, string_fns.rs, ...
│       ├── fs_fns.rs, path_fns.rs, process_fns.rs, os_fns.rs
│       ├── http_fns.rs, websocket_fns.rs, fetch_fns.rs, url_fns.rs
│       ├── buffer_fns.rs, intl_fns.rs, events_fns.rs, crypto_fns.rs
│       └── ...
├── objects/            # JS value types
│   ├── js_object.rs, js_array.rs, js_function.rs
│   ├── js_promise.rs, js_date.rs, js_proxy.rs
│   ├── js_collections.rs (Map, Set, WeakMap, WeakSet)
│   ├── js_typed_array.rs, js_string.rs, js_number.rs
│   └── safe_typed_array.rs
├── ffi/                # Foreign function interface
│   ├── mod.rs          # FFI entry points
│   ├── native.rs       # Native function registry
│   ├── safe_wrappers.rs    # SafePtr, SafeCStr, SafeSlice
│   └── safe_string.rs      # SafeFFIString, FFIStringBuffer
├── errors/             # Error types
│   ├── mod.rs
│   ├── runtime_errors.rs
│   └── type_errors.rs
└── cli/                # CLI subcommands
    ├── mod.rs
    └── build.rs        # Build subcommand implementation

modules/                # Workspace member crates
├── abi/                # Shared ABI types for native modules
├── native-macros/      # Proc macros for native module development
├── tails-validator/    # TypeScript validator
├── fs/                 # Pure Rust fs operations (feature-gated)
├── path/               # Pure Rust path operations (feature-gated)
├── process/            # Pure Rust process operations (feature-gated)
├── os/                 # Pure Rust os operations (feature-gated)
├── websocket/          # Pure Rust WebSocket client (feature-gated)
└── http/               # Pure Rust HTTP/1.1 server (feature-gated)

tests/
├── unit/               # Unit tests (parser, lexer, vm, minified)
├── integration/        # Integration tests
├── fixtures/           # TypeScript/CJS test fixtures
└── *.rs                # Feature-specific integration tests

benches/                # Criterion benchmarks
benchmarks/             # Benchmark suite with fixtures and runner
examples/               # Example TypeScript scripts
dist/                   # Build output for native modules
```

## Pipeline

1. **Lexer** (`compiler/lexer.rs`) — tokenizes TypeScript source
2. **Parser** (`compiler/parser/`) — produces AST
3. **Type Checker** (`compiler/type_checker/`) — optional type checking with narrowing
4. **Bytecode Generator** (`compiler/bytecode/`) — compiles AST to bytecode instructions
5. **VM / Interpreter** (`vm/interpreter/`) — stack-based bytecode interpreter with GC, exception handling, promises, and class support
6. **Runtime Environment** (`runtime_env/`) — 40+ native function modules providing Node.js-like APIs (fs, path, http, websocket, crypto, fetch, url, etc.)
7. **Native Module System** (`modules/`) — workspace crates compiled as dynamic libraries loaded at runtime via `libloading`
