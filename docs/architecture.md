```
src/
├── compiler/      # Lexer, parser, bytecode generator, type checker
├── vm/            # Virtual machine with interpreter and GC
│   └── interpreter/
│       ├── native_loader.rs  # Registry-based native module loader
│       └── ...
├── runtime_env/   # Native function adapters and async runtime
├── objects/       # JS value types (objects, arrays, functions, promises, proxies)
├── ffi/           # Foreign function interface
└── main.rs        # CLI entry point

modules/
├── abi/           # Shared ABI types for future dlopen support
├── fs/            # Pure Rust fs operations (feature-gated)
├── path/          # Pure Rust path operations (feature-gated)
├── process/       # Pure Rust process operations (feature-gated)
├── os/            # Pure Rust os operations (feature-gated)
├── websocket/     # Pure Rust WebSocket client operations (feature-gated)
└── http/          # Pure Rust HTTP/1.1 server primitives (feature-gated)
```
