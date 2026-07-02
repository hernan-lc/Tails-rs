use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};

/// Parse a `.env` file into a FxHashMap of key-value pairs.
///
/// Supports:
/// - Comments (lines starting with #)
/// - Empty lines
/// - Quoted values (single, double, or backtick)
/// - $VAR and ${VAR} expansion
/// - Inline comments after values
pub fn parse_env_file(content: &str) -> FxHashMap<String, String> {
    let mut vars = FxHashMap::default();
    for line in content.lines() {
        let line = line.trim();
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Find the first '='
        let eq_pos = match line.find('=') {
            Some(pos) => pos,
            None => continue,
        };
        let key = line[..eq_pos].trim().to_string();
        let raw_value = line[eq_pos + 1..].trim().to_string();

        if key.is_empty() {
            continue;
        }

        let value = unquote_and_expand(&raw_value, &vars);
        vars.insert(key, value);
    }
    vars
}

/// Remove surrounding quotes and expand $VAR / ${VAR} references.
fn unquote_and_expand(raw: &str, vars: &FxHashMap<String, String>) -> String {
    let unquoted = unquote(raw);
    expand_vars(&unquoted, vars)
}

/// Strip matching surrounding quotes (single, double, or backtick).
fn unquote(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.len() >= 2 {
        let first = trimmed.as_bytes()[0];
        let last = trimmed.as_bytes()[trimmed.len() - 1];
        if (first == b'"' && last == b'"')
            || (first == b'\'' && last == b'\'')
            || (first == b'`' && last == b'`')
        {
            return trimmed[1..trimmed.len() - 1].to_string();
        }
    }
    trimmed.to_string()
}

/// Expand `$VAR` and `${VAR}` references. Escape with `\$` to prevent expansion.
fn expand_vars(s: &str, vars: &FxHashMap<String, String>) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Escape: \$ -> literal $
        if bytes[i] == b'\\' && i + 1 < len && bytes[i + 1] == b'$' {
            result.push('$');
            i += 2;
            continue;
        }

        if bytes[i] == b'$' {
            i += 1;
            if i < len && bytes[i] == b'{' {
                // ${VAR} syntax
                i += 1;
                let start = i;
                while i < len && bytes[i] != b'}' {
                    i += 1;
                }
                let var_name = &s[start..i];
                if i < len {
                    i += 1; // skip '}'
                }
                if let Some(val) = vars.get(var_name) {
                    result.push_str(val);
                }
            } else if i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                // $VAR syntax
                let start = i;
                while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let var_name = &s[start..i];
                if let Some(val) = vars.get(var_name) {
                    result.push_str(val);
                }
            } else {
                result.push('$');
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

/// Find all `.env` files to load, walking up from `start_dir` to the filesystem root.
/// Returns paths in order of precedence (lowest first):
/// `.env` → `.env.{NODE_ENV}` → `.env.local`
pub fn find_env_files(start_dir: &Path, node_env: Option<&str>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let env_name = node_env.unwrap_or("development");

    // Walk up the directory tree looking for .env files
    let mut current = Some(start_dir);
    while let Some(dir) = current {
        // Precedence: .env < .env.{NODE_ENV} < .env.local
        let env_path = dir.join(".env");
        let env_node_path = dir.join(format!(".env.{}", env_name));
        let env_local_path = dir.join(".env.local");

        if env_path.exists() {
            candidates.push(env_path);
        }
        if env_node_path.exists() {
            candidates.push(env_node_path);
        }
        if env_local_path.exists() {
            candidates.push(env_local_path);
        }

        // If we found any .env files in this directory, stop walking up
        if !candidates.is_empty() {
            break;
        }

        current = dir.parent();
    }

    candidates
}

/// Load and apply `.env` files. Returns the number of variables loaded.
pub fn load_env_files(files: &[PathBuf]) -> usize {
    let mut loaded = FxHashMap::default();
    let mut count = 0;

    for file in files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let vars = parse_env_file(&content);
            for (key, value) in vars {
                loaded.insert(key, value);
            }
        }
    }

    // Apply to process environment (later files override earlier ones)
    for (key, value) in &loaded {
        std::env::set_var(key, value);
        count += 1;
    }

    count
}
