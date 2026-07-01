use std::env;
use std::path::{Component, Path, PathBuf};

use tails_native_macros::{tails_function, tails_module};

// ============================================================================
// Public API for direct Rust usage
// ============================================================================

pub const SEP: char = std::path::MAIN_SEPARATOR;
pub const DELIMITER: char = if cfg!(target_os = "windows") { ';' } else { ':' };

pub fn join(parts: &[String]) -> String {
    let mut result = PathBuf::new();
    for part in parts {
        result.push(part);
    }
    result.to_string_lossy().to_string()
}

pub fn resolve(parts: &[String]) -> String {
    let mut result = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    for part in parts {
        if Path::new(part).is_absolute() {
            result = part.clone();
        } else {
            result = Path::new(&result).join(part).to_string_lossy().to_string();
        }
    }

    normalize(&result)
}

pub fn basename(path: &str, ext: Option<&str>) -> String {
    let base = Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    if let Some(ext) = ext {
        if !ext.is_empty() && base.ends_with(ext) {
            let trim_len = base.len() - ext.len();
            return base[..trim_len].to_string();
        }
    }

    base
}

pub fn dirname(path: &str) -> String {
    Path::new(path)
        .parent()
        .map(|p| {
            let s = p.to_string_lossy().to_string();
            if s.is_empty() {
                ".".to_string()
            } else {
                s
            }
        })
        .unwrap_or_else(|| ".".to_string())
}

pub fn extname(path: &str) -> String {
    Path::new(path)
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default()
}

pub fn relative(from: &str, to: &str) -> String {
    let from_path = Path::new(from)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(from));
    let to_path = Path::new(to)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(to));

    let from_components: Vec<_> = from_path.components().collect();
    let to_components: Vec<_> = to_path.components().collect();

    let mut common_len = 0;
    for (a, b) in from_components.iter().zip(to_components.iter()) {
        if a == b {
            common_len += 1;
        } else {
            break;
        }
    }

    let mut parts: Vec<String> = Vec::new();
    for _ in common_len..from_components.len() {
        parts.push("..".to_string());
    }
    for comp in &to_components[common_len..] {
        parts.push(comp.as_os_str().to_string_lossy().to_string());
    }

    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join(std::path::MAIN_SEPARATOR_STR)
    }
}

pub fn is_absolute(path: &str) -> bool {
    Path::new(path).is_absolute()
}

pub fn normalize(path: &str) -> String {
    let mut components = Vec::new();
    let mut result = String::new();

    for component in Path::new(path).components() {
        match component {
            Component::Normal(c) => {
                let s = c.to_string_lossy().to_string();
                if s == ".." {
                    components.pop();
                } else if s != "." {
                    components.push(s);
                }
            }
            Component::ParentDir => {
                components.pop();
            }
            Component::CurDir => {}
            Component::RootDir => {
                result.push('/');
            }
            Component::Prefix(prefix) => {
                result.push_str(&prefix.as_os_str().to_string_lossy());
            }
        }
    }

    if result.is_empty() && !path.starts_with('/') {
        result = components.join("/");
        if result.is_empty() {
            result = ".".to_string();
        }
    } else {
        result.push_str(&components.join("/"));
    }

    result
}

// ============================================================================
// Native module (cdylib FFI exports)
// ============================================================================

#[tails_module(name = "tails-path")]
mod path_native {
    use super::*;
    #[tails_function]
    pub fn join(parts: String) -> String {
        let arr: Vec<String> = serde_json::from_str(&parts).unwrap_or_default();
        super::join(&arr)
    }

    #[tails_function]
    pub fn resolve(parts: String) -> String {
        let arr: Vec<String> = serde_json::from_str(&parts).unwrap_or_default();
        super::resolve(&arr)
    }

    #[tails_function]
    pub fn basename(path: String, ext: String) -> String {
        if ext.is_empty() {
            super::basename(&path, None)
        } else {
            super::basename(&path, Some(&ext))
        }
    }

    #[tails_function]
    pub fn dirname(path: String) -> String {
        super::dirname(&path)
    }

    #[tails_function]
    pub fn extname(path: String) -> String {
        super::extname(&path)
    }

    #[tails_function]
    pub fn filename(path: String) -> String {
        Path::new(&path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    #[tails_function]
    pub fn relative(from: String, to: String) -> String {
        super::relative(&from, &to)
    }

    #[tails_function]
    pub fn is_absolute(path: String) -> bool {
        super::is_absolute(&path)
    }

    #[tails_function]
    pub fn normalize(path: String) -> String {
        super::normalize(&path)
    }

    #[tails_function]
    pub fn sep() -> String {
        std::path::MAIN_SEPARATOR.to_string()
    }

    #[tails_function]
    pub fn delimiter() -> String {
        super::DELIMITER.to_string()
    }

    #[tails_function]
    pub fn parse(path: String) -> String {
        let p = Path::new(&path);
        let root = if p.is_absolute() {
            Some(std::path::MAIN_SEPARATOR.to_string())
        } else {
            None
        };
        let ext = p.extension().map(|s| format!(".{}", s.to_string_lossy()));
        let name = p.file_stem().map(|s| s.to_string_lossy().to_string());
        let dir = p.parent().map(|d| d.to_string_lossy().to_string());

        serde_json::json!({
            "root": root,
            "dir": dir,
            "base": p.file_name().map(|s| s.to_string_lossy().to_string()),
            "ext": ext,
            "name": name,
        }).to_string()
    }

    #[tails_function]
    pub fn format(parts: String) -> String {
        let obj: serde_json::Value = serde_json::from_str(&parts).unwrap_or(serde_json::Value::Null);
        let dir = obj.get("dir").and_then(|v| v.as_str()).unwrap_or("");
        let base = obj.get("base").and_then(|v| v.as_str()).unwrap_or("");
        let root = obj.get("root").and_then(|v| v.as_str()).unwrap_or("");

        if !dir.is_empty() && !base.is_empty() {
            let sep = std::path::MAIN_SEPARATOR;
            format!("{}{}{}", dir.trim_end_matches(sep), sep, base)
        } else if !root.is_empty() {
            format!("{}{}", root, base)
        } else {
            base.to_string()
        }
    }

    #[tails_function]
    pub fn contains(haystack: String, needle: String) -> bool {
        Path::new(&haystack).components().any(|c| {
            matches!(c, Component::Normal(s) if s.to_string_lossy() == needle)
        })
    }
}
