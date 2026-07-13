use std::path::{Path, PathBuf};
use anyhow::Result;

const TEST_HARNESS: &str = include_str!("../../tests/harness/embedded-harness.ts");

pub fn run(args: Vec<String>) -> Result<()> {
    let test_dir = if args.is_empty() {
        PathBuf::from("tests")
    } else {
        PathBuf::from(&args[0])
    };

    println!("🧪 Tails Test Runner");
    println!("====================\n");

    let test_files = discover_test_files(&test_dir);

    if test_files.is_empty() {
        println!("No test files found in '{}'", test_dir.display());
        return Ok(());
    }

    println!("Found {} test file(s)\n", test_files.len());

    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_skipped = 0;
    let mut had_errors = false;

    for file in &test_files {
        match run_test_file(file) {
            Ok(result) => {
                total_passed += result.passed;
                total_failed += result.failed;
                total_skipped += result.skipped;
            }
            Err(e) => {
                eprintln!("Error running {}: {}", file.display(), e);
                had_errors = true;
            }
        }
    }

    println!("\n================== TEST SUMMARY ==================");
    println!("Total: {} passed, {} failed, {} skipped", total_passed, total_failed, total_skipped);
    println!("==================================================");

    if total_failed > 0 || had_errors {
        std::process::exit(1);
    }

    Ok(())
}

struct TestResult {
    passed: u32,
    failed: u32,
    skipped: u32,
}

fn run_test_file(file: &Path) -> Result<TestResult> {
    let source = std::fs::read_to_string(file)?;

    let script = format!(
        r#"
        {}
        (async () => {{
            try {{
                {}
                await runTests();
            }} catch (e) {{
                console.log("Test file error: " + e.message);
                process.exit(1);
            }}
        }})();
        "#,
        TEST_HARNESS, source
    );

    let mut runtime = crate::TailsRuntime::default();

    let result = runtime.eval(&script);
    if let Err(_e) = result {
        return Ok(TestResult {
            passed: 0,
            failed: 1,
            skipped: 0,
        });
    }

    // Run the event loop for async tests
    if let Err(_e) = runtime.run_event_loop() {
        return Ok(TestResult {
            passed: 0,
            failed: 1,
            skipped: 0,
        });
    }

    // Default result for now
    Ok(TestResult {
        passed: 1,
        failed: 0,
        skipped: 0,
    })
}

fn discover_test_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_dir() {
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        files.extend(discover_test_files(&path));
                    } else if let Some(ext) = path.extension() {
                        let ext = ext.to_string_lossy();
                        if (ext == "ts" || ext == "js") && is_test_file(&path) {
                            files.push(path);
                        }
                    }
                }
            }
            Err(e) => eprintln!("Warning: Could not read directory {}: {}", dir.display(), e),
        }
    }
    files.sort();
    files
}

fn is_test_file(path: &Path) -> bool {
    let name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent = path.parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    name.starts_with("test") 
        || name.ends_with(".test.ts") 
        || name.ends_with(".test.js")
        || name.ends_with(".spec.ts") 
        || name.ends_with(".spec.js")
        || parent == "test"
        || parent == "tests"
}
