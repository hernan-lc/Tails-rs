use std::fs;
use std::path::{Path, PathBuf};
use tails::TailsRuntime;

struct TestMetadata {
    description: String,
    negative_type: Option<String>,
}

fn parse_metadata(content: &str) -> TestMetadata {
    let mut meta = TestMetadata {
        description: String::new(),
        negative_type: None,
    };

    if let Some(start) = content.find("/*---") {
        let content_after_start = &content[start + 5..];
        if let Some(end) = content_after_start.find("---*/") {
            let frontmatter = &content_after_start[..end];
            let mut in_negative = false;
            for line in frontmatter.lines() {
                let trimmed = line.trim();
                if let Some(stripped) = trimmed.strip_prefix("description:") {
                    meta.description = stripped.trim().to_string();
                } else if trimmed.starts_with("negative:") {
                    in_negative = true;
                } else if in_negative && trimmed.starts_with("type:") {
                    meta.negative_type = Some(
                        trimmed["type:".len()..]
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string(),
                    );
                } else if let Some(stripped) = trimmed.strip_prefix("negative.type:") {
                    meta.negative_type = Some(
                        stripped
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string(),
                    );
                }
            }
        }
    }
    meta
}

fn collect_test_files(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_test_files(&path, files)?;
            } else {
                if let Some(ext) = path.extension() {
                    if (ext == "js" || ext == "ts") && !path.to_string_lossy().contains("harness") {
                        files.push(path);
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn run_conformance_suite() {
    let harness_path = Path::new("tests/conformance/harness/assert.js");
    let harness_code = fs::read_to_string(harness_path)
        .expect("Failed to read tests/conformance/harness/assert.js");

    let conformance_dir = Path::new("tests/conformance");
    let mut test_files = Vec::new();
    collect_test_files(conformance_dir, &mut test_files).expect("Failed to collect test files");

    assert!(!test_files.is_empty(), "No conformance test files found!");

    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    println!("Running {} conformance tests...", test_files.len());

    for test_file in &test_files {
        let content = fs::read_to_string(test_file)
            .unwrap_or_else(|_| panic!("Failed to read test file {:?}", test_file));
        let meta = parse_metadata(&content);

        let mut runtime = TailsRuntime::default();

        // Inject harness
        if let Err(e) = runtime.eval(&harness_code) {
            panic!("Failed to inject harness code for {:?}: {:?}", test_file, e);
        }

        // Run the test
        let mut result = runtime.eval(&content);
        if result.is_ok() {
            if let Err(e) = runtime.run_event_loop() {
                result = Err(e);
            }
        }

        let test_name = test_file
            .strip_prefix(conformance_dir)
            .unwrap_or(test_file)
            .to_string_lossy()
            .to_string();

        match (result, meta.negative_type) {
            (Ok(_), None) => {
                passed += 1;
            }
            (Ok(_), Some(expected_err)) => {
                failed += 1;
                failures.push(format!(
                    "{} failed: Expected to throw error of type '{}', but succeeded.",
                    test_name, expected_err
                ));
            }
            (Err(e), None) => {
                failed += 1;
                failures.push(format!(
                    "{} failed: Expected success, but threw {}: {}",
                    test_name,
                    e.kind_name(),
                    e.message()
                ));
            }
            (Err(e), Some(expected_err)) => {
                let actual_err = e.kind_name();
                if actual_err == expected_err {
                    passed += 1;
                } else {
                    failed += 1;
                    failures.push(format!(
                        "{} failed: Expected to throw error of type '{}', but threw '{}' instead: {}",
                        test_name, expected_err, actual_err, e.message()
                    ));
                }
            }
        }
    }

    println!("\nConformance Test Run Summary:");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);

    if failed > 0 {
        for failure in &failures {
            eprintln!("  - {}", failure);
        }
        panic!("{} conformance tests failed!", failed);
    }
}
