use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant, SystemTime};

fn print_usage() {
    eprintln!("Usage: tails [OPTIONS] <script.ts>");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --watch         Watch for file changes and re-run automatically");
    eprintln!("  --color         Enable colored output (default)");
    eprintln!("  --no-color      Disable colored output");
    eprintln!("  --timestamps    Show timestamps in console output");
    eprintln!("  --help          Show this help message");
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
            eprintln!("[tails] Finished in {}ms.", elapsed.as_millis());
        }
        Err(e) => {
            let file_str = script_path.to_string_lossy().to_string();
            let e_with_file = e.with_file(file_str);
            eprint!("{}", e_with_file.display(Some(&source)));
            eprintln!("[tails] Failed in {}ms.", elapsed.as_millis());
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

    let watch_mode = args.iter().any(|a| a == "--watch" || a == "-w");
    let no_color = args.iter().any(|a| a == "--no-color");
    let timestamps = args.iter().any(|a| a == "--timestamps");

    // Set color and timestamp preferences
    tails::runtime_env::native_fns::console::set_colors(!no_color);
    tails::runtime_env::native_fns::console::set_timestamps(timestamps);

    let script_arg = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .context("No script file specified. Run 'tails --help' for usage.")?;

    let script_path = PathBuf::from(script_arg);
    if !script_path.exists() {
        eprintln!("Error: File '{}' not found", script_arg);
        std::process::exit(1);
    }

    if watch_mode {
        watch(&script_path)?;
    } else {
        run_script(&script_path)?;
    }

    Ok(())
}
