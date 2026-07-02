## CLI

```bash
# Run a script
tails run script.ts

# Run with file watching
tails run --watch script.ts

# Run with custom env file
tails run --env-file .env.production script.ts

# Run without auto-loading .env files
tails run --no-env-file script.ts

# Run with colored output (default) or without
tails run --color script.ts
tails run --no-color script.ts

# Run with timestamps in console output
tails run --timestamps script.ts

# Build a native module
tails build -p my-module --release

# Clean the dist/ directory
tails clean
```

## As a Library

```rust
use tails::{TailsRuntime, Value};

let mut runtime = TailsRuntime::default();
let result = runtime.eval_module(&source, &path)?;
```

### Runtime Configuration

```rust
use tails::{RuntimeConfig, TailsRuntime};

let config = RuntimeConfig::default();
let mut runtime = TailsRuntime::with_config(config);
```

## Dotenv Support

Tails-rs automatically loads `.env` files from the script's directory:

1. `.env` — always loaded if present
2. `.env.{NODE_ENV}` — loaded when `NODE_ENV` is set (e.g., `.env.production`)
3. `.env.local` — loaded last, overrides others

Use `--env-file <path>` to load a specific file, or `--no-env-file` to disable auto-loading.
