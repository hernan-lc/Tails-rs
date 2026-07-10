use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant, SystemTime};

fn print_usage() {
    eprintln!("Usage: tails <command> [OPTIONS]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  run <script.ts>       Run a TypeScript script (default)");
    eprintln!("  build [OPTIONS]       Build native module to dist/");
    eprintln!("  clean                 Remove dist/ directory");
    eprintln!();
    eprintln!("Run options:");
    eprintln!("  --watch               Watch for file changes and re-run automatically");
    eprintln!("  --env-file <path>     Load environment variables from a specific .env file");
    eprintln!("  --no-env-file         Disable automatic .env file loading");
    eprintln!("  --color               Enable colored output (default)");
    eprintln!("  --no-color            Disable colored output");
    eprintln!("  --timestamps          Show timestamps in console output");
    eprintln!();
    eprintln!("Build options:");
    eprintln!("  --package, -p <name>  Package to build (auto-detects cdylib if omitted)");
    eprintln!("  --release             Build in release mode");
    eprintln!("  --target-dir <path>   Custom target directory");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  tails run script.ts");
    eprintln!("  tails build --package my-module --release");
    eprintln!("  tails clean");
}

fn now_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}

fn discover_imports(script_path: &Path) -> Vec<PathBuf> {
    let source = match std::fs::read_to_string(script_path) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let base = script_path.parent().unwrap_or(Path::new("."));
    let mut imports = Vec::new();
    let mut seen = HashSet::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }

        // import ... from "./path"
        // import "./path"
        // export ... from "./path"
        // require("./path")
        for pattern in [
            "from \"",
            "from '",
            "import \"",
            "import '",
            "require(\"",
            "require('",
        ] {
            if let Some(idx) = trimmed.find(pattern) {
                let rest = &trimmed[idx + pattern.len()..];
                if let Some(end) = rest.find('"').or_else(|| rest.find('\'')) {
                    let import_path = &rest[..end];
                    if import_path.starts_with("./") || import_path.starts_with("../") {
                        let resolved = base.join(import_path);
                        let canonical = resolved.canonicalize().unwrap_or(resolved.clone());
                        if !seen.contains(&canonical) {
                            seen.insert(canonical.clone());
                            imports.push(canonical);
                            // Recurse into imports (one level)
                            let sub = discover_imports(&resolved);
                            for s in sub {
                                if !seen.contains(&s) {
                                    seen.insert(s.clone());
                                    imports.push(s);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    imports
}

fn run_script(script_path: &Path) -> Result<()> {
    // Ensure the shared handle registry symbols are preserved and exported.
    let _ = tails_abi::tails_abi_get_shared_registry();

    // So Ctrl+C / SIGTERM stop the event loop cleanly (normal exit code)
    // instead of an uncaught signal that shells report as "terminated by SIGTERM".
    tails::runtime_env::native_fns::process_fns::install_signal_handlers();

    let source = std::fs::read_to_string(script_path)
        .with_context(|| format!("Failed to read '{}'", script_path.display()))?;

    let start = Instant::now();
    let mut runtime = tails::TailsRuntime::default();
    let result = runtime.eval_module(&source, script_path);
    let elapsed = start.elapsed();

    match result {
        Ok(value) => {
            if !matches!(value, tails::Value::Undefined) {
                println!("{:?}", value);
            }
            eprintln!("[tails] Script finished in {}ms.", elapsed.as_millis());

            // If the script registered long-lived event sources (HTTP servers,
            // TCP connections, timers, …) keep the process alive by running
            // the cooperative event loop — just like Node.js or Bun.
            if runtime.has_pending_work() {
                eprintln!("[tails] Event loop running (Ctrl+C to exit).");
                if let Err(e) = runtime.run_event_loop() {
                    eprintln!("[tails] Event loop error: {}", e);
                }
            }
        }
        Err(e) => {
            let file_str = script_path.to_string_lossy().to_string();
            let e_with_file = e.with_file(file_str);
            eprint!("{}", e_with_file.display(Some(&source)));
            eprintln!("[tails] Failed in {}ms.", elapsed.as_millis());
            std::process::exit(1);
        }
    }

    Ok(())
}

fn watch(script_path: &Path) -> Result<()> {
    type FileEvent = Result<Event, notify::Error>;
    let (tx, rx): (mpsc::Sender<FileEvent>, Receiver<FileEvent>) = mpsc::channel();

    let mut watcher: RecommendedWatcher = Watcher::new(
        tx,
        notify::Config::default().with_poll_interval(Duration::from_millis(100)),
    )?;

    // Watch main file
    watcher.watch(script_path, RecursiveMode::NonRecursive)?;

    // Watch discovered imports
    let imports = discover_imports(script_path);
    for import_path in &imports {
        if import_path.exists() {
            let _ = watcher.watch(import_path, RecursiveMode::NonRecursive);
        }
    }

    let import_count = imports.len();
    eprintln!(
        "[tails] Watching '{}' (+{} imports) for changes... (Ctrl+C to exit)",
        script_path.display(),
        import_count
    );

    // Initial run
    clear_screen();
    eprintln!(
        "\x1B[36m[tails] Running {} at {}\x1B[0m",
        script_path.display(),
        now_timestamp()
    );
    let _ = run_script(script_path);

    let mut last_trigger = Instant::now();
    let debounce = Duration::from_millis(300);

    for res in rx {
        match res {
            Ok(event) => {
                let relevant = matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                );
                if !relevant {
                    continue;
                }

                // Only react to changes on watched files
                let watched = event
                    .paths
                    .iter()
                    .any(|p| p == script_path || imports.iter().any(|i| i == p));
                if !watched {
                    continue;
                }

                // Debounce
                let now = Instant::now();
                if now.duration_since(last_trigger) < debounce {
                    continue;
                }
                last_trigger = now;

                clear_screen();
                eprintln!(
                    "\x1B[36m[tails] Re-running {} at {} (change detected)\x1B[0m",
                    script_path.display(),
                    now_timestamp()
                );
                let _ = run_script(script_path);
            }
            Err(e) => {
                eprintln!("\x1B[31m[tails] Watch error: {:?}\x1B[0m", e);
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        std::process::exit(0);
    }

    // Check for subcommands
    let first_arg = args.first().map(|s| s.as_str());

    match first_arg {
        Some("build") => {
            let build_args: Vec<String> = args[1..].to_vec();
            let mut package = None;
            let mut release = false;
            let mut target_dir = None;

            let mut i = 0;
            while i < build_args.len() {
                match build_args[i].as_str() {
                    "--package" | "-p" => {
                        i += 1;
                        package = build_args.get(i).cloned();
                    }
                    "--release" => release = true,
                    "--target-dir" => {
                        i += 1;
                        target_dir = build_args.get(i).cloned();
                    }
                    _ => {}
                }
                i += 1;
            }

            let opts = tails::cli::build::BuildOptions {
                package,
                release,
                target_dir,
            };
            tails::cli::build::run_build(opts)?;
            return Ok(());
        }
        Some("clean") => {
            tails::cli::build::run_clean()?;
            return Ok(());
        }
        Some("run") => {
            // Fall through to run mode with remaining args
        }
        Some(arg) if arg.ends_with(".ts") => {
            // Script file directly (backward compatible)
        }
        Some(other) => {
            eprintln!("Unknown command: {}", other);
            eprintln!("Run 'tails --help' for usage.");
            std::process::exit(1);
        }
        None => {
            eprintln!("No command specified. Run 'tails --help' for usage.");
            std::process::exit(1);
        }
    }

    // Determine args for run mode (skip "run" subcommand if present)
    let run_args: Vec<String> = if first_arg == Some("run") {
        args[1..].to_vec()
    } else {
        args
    };

    let watch_mode = run_args.iter().any(|a| a == "--watch" || a == "-w");
    let no_color = run_args.iter().any(|a| a == "--no-color");
    let timestamps = run_args.iter().any(|a| a == "--timestamps");
    let no_env_file = run_args.iter().any(|a| a == "--no-env-file");

    // Find custom --env-file argument
    let custom_env_file = run_args.windows(2).find_map(|w| {
        if w[0] == "--env-file" {
            Some(w[1].clone())
        } else {
            None
        }
    });

    // Set color and timestamp preferences
    tails::runtime_env::native_fns::console::set_colors(!no_color);
    tails::runtime_env::native_fns::console::set_timestamps(timestamps);

    let script_arg = run_args
        .iter()
        .enumerate()
        .filter(|(i, a)| {
            // Skip flags that take a value
            if **a == "--env-file" {
                return false;
            }
            // Skip the value after --env-file
            if *i > 0 && run_args.get(i - 1).map(|a| a.as_str()) == Some("--env-file") {
                return false;
            }
            !a.starts_with('-')
        })
        .map(|(_, a)| a)
        .next()
        .context("No script file specified. Run 'tails --help' for usage.")?;

    let script_path = PathBuf::from(script_arg);
    if !script_path.exists() {
        eprintln!("Error: File '{}' not found", script_arg);
        std::process::exit(1);
    }

    // Load .env files before running the script
    if !no_env_file {
        if let Some(ref custom_path) = custom_env_file {
            // Load a specific .env file
            let path = PathBuf::from(custom_path);
            if path.exists() {
                let count = tails::dotenv::load_env_files(std::slice::from_ref(&path));
                eprintln!(
                    "[tails] Loaded {} env variables from {}",
                    count,
                    path.display()
                );
            } else {
                eprintln!("[tails] Warning: env file '{}' not found", custom_path);
            }
        } else {
            // Auto-load .env from script's directory
            let script_dir = script_path.parent().unwrap_or_else(|| Path::new("."));
            let node_env = std::env::var("NODE_ENV").ok();
            let env_files = tails::dotenv::find_env_files(script_dir, node_env.as_deref());
            if !env_files.is_empty() {
                let count = tails::dotenv::load_env_files(&env_files);
                for f in &env_files {
                    eprintln!("[tails] Loaded env from {}", f.display());
                }
                eprintln!("[tails] {} env variables loaded", count);
            }
        }
    }

    if watch_mode {
        watch(&script_path)?;
    } else {
        run_script(&script_path)?;
    }

    Ok(())
}
