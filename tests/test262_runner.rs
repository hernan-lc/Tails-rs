use std::fs;
use std::path::{Path, PathBuf};
use tails::TailsRuntime;
use fancy_regex::Regex;

#[derive(Debug, Default)]
struct Test262Metadata {
    #[allow(dead_code)]
    flags: Vec<String>,
    includes: Vec<String>,
    negative: Option<NegativeMetadata>,
}

#[derive(Debug)]
struct NegativeMetadata {
    #[allow(dead_code)]
    phase: String,
    type_name: String,
}

struct Test262Runner {
    harness_path: PathBuf,
}

impl Test262Runner {
    fn new(harness_path: PathBuf) -> Self {
        Self { harness_path }
    }

    fn parse_metadata(&self, source: &str) -> Test262Metadata {
        let mut metadata = Test262Metadata::default();

        // Find the frontmatter block between /*--- and ---*/
        let re_block = Regex::new(r"(?s)/\*---\s*(.*?)\s*---\*/").unwrap();
        if let Ok(Some(caps)) = re_block.captures(source) {
            let inner = caps.get(1).unwrap().as_str();

            // Simple line-based parser for YAML subset
            for line in inner.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("flags: [") {
                    metadata.flags = trimmed[8..trimmed.len()-1].split(',').map(|s| s.trim().to_string()).collect();
                } else if trimmed.starts_with("includes: [") {
                    metadata.includes = trimmed[11..trimmed.len()-1].split(',').map(|s| s.trim().to_string()).collect();
                }
            }

            // Fallback for negative if simple parsing failed
            let re_neg = Regex::new(r"negative:\s+phase: (\w+)\s+type: (\w+)").unwrap();
            if let Ok(Some(caps)) = re_neg.captures(inner) {
                metadata.negative = Some(NegativeMetadata {
                    phase: caps.get(1).unwrap().as_str().to_string(),
                    type_name: caps.get(2).unwrap().as_str().to_string(),
                });
            }
        }

        metadata
    }

    fn run_test(&self, test_path: &Path) -> Result<(), String> {
        let source = fs::read_to_string(test_path).map_err(|e| e.to_string())?;
        let metadata = self.parse_metadata(&source);

        let mut rt = TailsRuntime::default();

        // Register $262 and print
        rt.eval("globalThis.print = console.log;").map_err(|e| e.to_string())?;
        rt.eval("globalThis.$262 = { gc: function() {}, detachArrayBuffer: function() {}, evalScript: function(s) { return eval(s); } };").map_err(|e| e.to_string())?;

        // Load harness files
        for include in &metadata.includes {
            let include_path = self.harness_path.join(include);
            let include_source = fs::read_to_string(&include_path).map_err(|e| format!("Failed to load harness {}: {}", include_path.display(), e))?;
            rt.eval(&include_source).map_err(|e| format!("Failed to eval harness {}: {}", include, e))?;
        }

        // Run the actual test
        let result = rt.eval(&source);

        match (result, metadata.negative) {
            (Ok(_), None) => Ok(()),
            (Err(_), Some(_)) => Ok(()),
            (Ok(_), Some(neg)) => Err(format!("Expected test to fail with {}, but it passed", neg.type_name)),
            (Err(e), None) => Err(format!("Test failed: {}", e)),
        }
    }
}

// Redactor: Filters and formats test results for comparison
#[allow(dead_code)]
struct Redactor {
    skip_list: Vec<String>,
}

impl Redactor {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            skip_list: vec![
                "async-functions".to_string(),
            ],
        }
    }

    #[allow(dead_code)]
    fn should_skip(&self, name: &str) -> bool {
        self.skip_list.iter().any(|s| name.contains(s))
    }

    #[allow(dead_code)]
    fn redact_output(&self, output: &str) -> String {
        // Redact absolute paths
        let re_path = Regex::new(r"/[^ ]+/").unwrap();
        re_path.replace_all(output, "[REDACTED_PATH]/").to_string()
    }
}

#[test]
fn test_test262_basic_run() {
    let runner = Test262Runner::new(PathBuf::from("tests/fixtures/test262/harness"));
    let redactor = Redactor::new();
    let test_path = Path::new("tests/fixtures/test262/test/basic.js");

    if test_path.exists() && !redactor.should_skip("basic.js") {
        let result = runner.run_test(test_path);
        if let Err(e) = &result {
            println!("Error: {}", redactor.redact_output(e));
        }
        assert!(result.is_ok());
    }
}

#[test]
fn test_test262_negative_test() {
    let runner = Test262Runner::new(PathBuf::from("tests/fixtures/test262/harness"));

    let source = r#"/*---
negative:
  phase: runtime
  type: ReferenceError
---*/
nonExistentVariable;"#;

    let temp_dir = std::env::temp_dir();
    let test_path = temp_dir.join("test262_negative.js");
    fs::write(&test_path, source).unwrap();

    assert!(runner.run_test(&test_path).is_ok());
    let _ = fs::remove_file(test_path);
}
