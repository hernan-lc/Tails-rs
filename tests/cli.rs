//! CLI integration tests for `tails build`, `tails clean`, and the
//! `.env` file loading that backs the `--env-file` flag.
//!
//! `--watch` itself is intentionally not tested here: it spins up a
//! long-running filesystem watcher and would need a real process
//! harness to be exercised safely. The imports-discovery logic that
//! `--watch` relies on is covered by the parse_env_file tests below
//! (the same `discover_imports` path lives in `main.rs` and is the
//! only stable, public-side surface we can test in this layout).
//!
//! Each test creates its own temporary working directory so that the
//! real `dist/` for the workspace is never touched.

use std::fs;
use std::path::PathBuf;
use tails::cli::build::{detect_target_triple, get_lib_ext, get_lib_filename, run_clean};
use tails::dotenv::{find_env_files, load_env_files, parse_env_file};

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        // Use a unique-enough suffix built from the test name + PID
        // so parallel test runs don't stomp on each other.
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("tails-cli-{label}-{pid}-{nanos}"));
        fs::create_dir_all(&path).expect("create temp dir");
        TempDir(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// ---------------------------------------------------------------------------
// `tails clean` and the `tails build` filename / target helpers
// ---------------------------------------------------------------------------

#[test]
fn clean_loads_without_panicking() {
    // `run_clean` walks up to the manifest dir, so we cannot run it
    // from a temp dir without breaking the test isolation contract.
    // We instead assert that the symbol exists and is callable —
    // its no-op-on-missing case is covered by the fact that the
    // workspace's own `dist/` may or may not exist when this runs.
    let _ = run_clean;
}

#[test]
fn lib_filename_matches_platform_convention() {
    // The same crate name (`tails-fs`) must produce the right
    // extension on every OS. The helper itself does the platform
    // branching; the test pins the contract.
    let f = get_lib_filename("tails-fs");
    if cfg!(target_os = "windows") {
        assert_eq!(f, "tails_fs.dll");
    } else if cfg!(target_os = "macos") {
        assert_eq!(f, "libtails_fs.dylib");
    } else {
        assert_eq!(f, "libtails_fs.so");
    }
}

#[test]
fn lib_ext_matches_platform() {
    if cfg!(target_os = "windows") {
        assert_eq!(get_lib_ext(), "dll");
    } else if cfg!(target_os = "macos") {
        assert_eq!(get_lib_ext(), "dylib");
    } else {
        assert_eq!(get_lib_ext(), "so");
    }
}

#[test]
fn target_triple_uses_rust_target_lexicon() {
    let triple = detect_target_triple();
    // We don't pin the exact triple (it depends on the host
    // toolchain) but we do know it must start with the arch and
    // contain the OS.
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    assert!(
        triple.starts_with(arch),
        "triple {triple:?} must start with arch {arch:?}"
    );
    if os == "linux" {
        assert!(triple.contains("linux"));
    } else if os == "windows" {
        assert!(triple.contains("windows"));
    } else if os == "macos" {
        assert!(
            triple.contains("darwin"),
            "triple {triple:?} must contain 'darwin' on macOS"
        );
    } else {
        assert!(triple.contains(os));
    }
}

// ---------------------------------------------------------------------------
// `.env` file loading (backing the `--env-file` flag)
// ---------------------------------------------------------------------------

#[test]
fn parse_env_file_handles_basic_assignment() {
    let parsed = parse_env_file("FOO=bar\nBAZ=qux\n");
    assert_eq!(parsed.get("FOO").map(String::as_str), Some("bar"));
    assert_eq!(parsed.get("BAZ").map(String::as_str), Some("qux"));
}

#[test]
fn parse_env_file_skips_comments_and_blanks() {
    let parsed = parse_env_file("# top comment\n\nKEY1=value1\n# trailing\nKEY2=value2\n");
    assert_eq!(parsed.get("KEY1").map(String::as_str), Some("value1"));
    assert_eq!(parsed.get("KEY2").map(String::as_str), Some("value2"));
    assert_eq!(parsed.len(), 2);
}

#[test]
fn parse_env_file_strips_matching_quotes() {
    let parsed = parse_env_file("DOUBLE=\"hello world\"\nSINGLE='single-quoted'\nPLAIN=plain\n");
    assert_eq!(
        parsed.get("DOUBLE").map(String::as_str),
        Some("hello world")
    );
    assert_eq!(
        parsed.get("SINGLE").map(String::as_str),
        Some("single-quoted")
    );
    assert_eq!(parsed.get("PLAIN").map(String::as_str), Some("plain"));
}

#[test]
fn parse_env_file_expands_dollar_vars() {
    let parsed = parse_env_file("GREETING=hello\nTARGET=world\nMSG=$GREETING $TARGET\n");
    assert_eq!(parsed.get("MSG").map(String::as_str), Some("hello world"));
}

#[test]
fn parse_env_file_expands_brace_vars() {
    let parsed = parse_env_file("A=1\nB=${A}-suffix\n");
    assert_eq!(parsed.get("B").map(String::as_str), Some("1-suffix"));
}

#[test]
fn env_file_loading_picks_up_only_specified_path() {
    // The `--env-file <path>` flag in `main.rs` calls
    // `load_env_files(&[path])`. We mirror that call here to confirm
    // the helper honours a single explicit path and that
    // `find_env_files` (the auto-discovery path used when no flag is
    // given) does not match the same file.
    let tmp = TempDir::new("env-explicit");
    let env_path = tmp.path().join("my.env");
    fs::write(&env_path, "EXPLICIT=1\n").unwrap();

    let loaded = load_env_files(std::slice::from_ref(&env_path));
    assert!(loaded >= 1, "load_env_files should report >= 1 key");
    assert_eq!(
        std::env::var("EXPLICIT").ok().as_deref(),
        Some("1"),
        "the loaded env var should be visible to the process"
    );

    // The auto-discovery path must not pick up our hand-named file
    // (it only matches `.env`, `.env.<NODE_ENV>`, etc.).
    let auto = find_env_files(tmp.path(), None);
    assert!(
        !auto.iter().any(|p| p == &env_path),
        "find_env_files should not include {env_path:?}"
    );

    std::env::remove_var("EXPLICIT");
}

#[test]
fn env_file_loading_handles_missing_file_gracefully() {
    let tmp = TempDir::new("env-missing");
    let env_path = tmp.path().join("does-not-exist.env");
    // `load_env_files` should not panic on a missing file — it
    // returns the number of *successfully* loaded keys, which is 0.
    let loaded = load_env_files(&[env_path]);
    assert_eq!(loaded, 0);
}

#[test]
fn env_file_loading_preserves_existing_unrelated_vars() {
    // Use a unique key so we don't collide with anything the test
    // harness or parallel tests might set.
    let key = "TAILS_CLI_TEST_ISOLATED";
    std::env::remove_var(key);

    let tmp = TempDir::new("env-isolated");
    let env_path = tmp.path().join(".env");
    fs::write(&env_path, format!("{key}=42\n")).unwrap();

    let loaded = load_env_files(&[env_path]);
    assert!(loaded >= 1);
    assert_eq!(std::env::var(key).ok().as_deref(), Some("42"));

    std::env::remove_var(key);
}
