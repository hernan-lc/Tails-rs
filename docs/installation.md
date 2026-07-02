## Installation

```bash
cargo build --release
```

### Build with Feature Flags

Native modules (`fs`, `path`, `process`, `os`, `websocket`, `http`) are compiled as optional Cargo features, all enabled by default.

```bash
# All modules (default)
cargo build --release

# Without any native modules (smallest binary)
cargo build --release --no-default-features

# Only fs and path
cargo build --release --no-default-features -F fs -F path

# Everything except os and websocket
cargo build --release --no-default-features -F fs -F path -F process -F http

# Only http server
cargo build --release --no-default-features -F http
```

### Available Features

| Feature | Module | Description |
|---------|--------|-------------|
| `fs` | `tails-fs` | File system operations (read, write, stat, mkdir, etc.) |
| `path` | `tails-path` | Path manipulation (join, resolve, basename, etc.) |
| `process` | `tails-process` | Process info and control (env, argv, exit, etc.) |
| `os` | `tails-os` | OS information (platform, arch, cpus, memory, etc.) |
| `websocket` | `tails-websocket` | WebSocket client operations |
| `http` | `tails-http` | HTTP/1.1 server primitives |
