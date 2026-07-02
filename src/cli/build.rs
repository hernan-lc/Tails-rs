use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct BuildOptions {
    pub package: Option<String>,
    pub release: bool,
    pub target_dir: Option<String>,
}

pub fn detect_target_triple() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let env = if cfg!(target_env = "musl") {
        "musl"
    } else if cfg!(target_env = "gnu") {
        "gnu"
    } else {
        ""
    };

    match (arch, os) {
        ("x86_64", "linux") => format!(
            "x86_64-unknown-linux-{}",
            if env.is_empty() { "gnu" } else { env }
        ),
        ("aarch64", "linux") => format!(
            "aarch64-unknown-linux-{}",
            if env.is_empty() { "gnu" } else { env }
        ),
        ("x86_64", "macos") => "x86_64-apple-darwin".to_string(),
        ("aarch64", "macos") => "aarch64-apple-darwin".to_string(),
        ("x86_64", "windows") => "x86_64-pc-windows-msvc".to_string(),
        _ => format!("{}-unknown-{}", arch, os),
    }
}

pub fn get_lib_filename(crate_name: &str) -> String {
    let name = crate_name.replace('-', "_");
    match std::env::consts::OS {
        "windows" => format!("{}.dll", name),
        "macos" => format!("lib{}.dylib", name),
        _ => format!("lib{}.so", name),
    }
}

pub fn get_lib_ext() -> &'static str {
    match std::env::consts::OS {
        "windows" => "dll",
        "macos" => "dylib",
        _ => "so",
    }
}

pub fn run_build(opts: BuildOptions) -> Result<()> {
    let manifest_dir = find_manifest_dir()?;
    let workspace_root = &manifest_dir;

    // Find Cargo.toml to get package name
    let cargo_toml = fs::read_to_string(workspace_root.join("Cargo.toml"))
        .context("Failed to read Cargo.toml")?;

    let package_name = if let Some(ref pkg) = opts.package {
        // If it looks like a path, resolve the package name from Cargo.toml
        if pkg.contains('/') || pkg.contains('\\') || pkg.ends_with(".toml") {
            let pkg_path = if pkg.ends_with(".toml") {
                Path::new(pkg).parent().unwrap_or(Path::new("."))
            } else {
                Path::new(pkg)
            };
            let pkg_dir = workspace_root.join(pkg_path);
            let pkg_toml = pkg_dir.join("Cargo.toml");
            if pkg_toml.exists() {
                let toml_content = fs::read_to_string(&pkg_toml)
                    .with_context(|| format!("Failed to read {}", pkg_toml.display()))?;
                extract_package_name(&toml_content)
                    .with_context(|| format!("No [package] name in {}", pkg_toml.display()))?
            } else {
                anyhow::bail!("No Cargo.toml found at {}", pkg_dir.display());
            }
        } else {
            pkg.clone()
        }
    } else {
        // Auto-detect: find the first cdylib crate in workspace
        find_cdylib_package(&cargo_toml, workspace_root)?
    };

    eprintln!("[tails] Building package: {}", package_name);

    // Run cargo build
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    if opts.release {
        cmd.arg("--release");
    }
    if let Some(ref target) = opts.target_dir {
        cmd.arg("--target-dir").arg(target);
    }
    cmd.arg("-p").arg(&package_name);

    let status = cmd
        .current_dir(workspace_root)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    // Determine output paths
    let target_dir = opts
        .target_dir
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("target"));

    let profile = if opts.release { "release" } else { "debug" };
    let lib_filename = get_lib_filename(&package_name);
    let src_path = target_dir.join(profile).join(&lib_filename);

    if !src_path.exists() {
        anyhow::bail!(
            "Library not found at {}. Build may have failed.",
            src_path.display()
        );
    }

    // Create dist directory
    let dist_dir = workspace_root.join("dist");
    fs::create_dir_all(&dist_dir).context("Failed to create dist directory")?;

    // Copy library
    let dst_path = dist_dir.join(&lib_filename);
    fs::copy(&src_path, &dst_path).context("Failed to copy library to dist/")?;
    eprintln!("[tails] Output: {}", dst_path.display());

    // Also copy under a module-name alias so that `import x from
    // "x.native"` works for any package following the `tails-<module>`
    // naming convention. Without this, the runtime's loader would search
    // for `lib<x>.so` in dist/, but cargo produces `lib<package>.so`.
    if let Some(stripped) = package_name.strip_prefix("tails-") {
        let alias_name = format!("lib{}.{}", stripped, get_lib_ext());
        let alias_path = dist_dir.join(&alias_name);
        if dst_path != alias_path && fs::copy(&dst_path, &alias_path).is_ok() {
            eprintln!("[tails] Alias: {}", alias_path.display());
        }
    }

    // Generate or copy .d.ts
    let dts_path = dist_dir.join(format!("{}.d.ts", package_name));

    // Check if there's a .d.ts file in the package directory
    let pkg_dir = workspace_root.join("modules").join(&package_name);
    let pkg_dts = pkg_dir.join(format!("{}.d.ts", package_name));
    let alt_pkg_dts = pkg_dir.join("index.d.ts");

    if pkg_dts.exists() {
        fs::copy(&pkg_dts, &dts_path)
            .with_context(|| format!("Failed to copy {} to dist/", pkg_dts.display()))?;
        eprintln!("[tails] TypeScript definitions: {}", dts_path.display());
    } else if alt_pkg_dts.exists() {
        fs::copy(&alt_pkg_dts, &dts_path)
            .with_context(|| format!("Failed to copy {} to dist/", alt_pkg_dts.display()))?;
        eprintln!("[tails] TypeScript definitions: {}", dts_path.display());
    } else {
        // Generate from library metadata
        let dts_content = generate_dts(&src_path, &package_name)?;
        fs::write(&dts_path, &dts_content).context("Failed to write .d.ts file")?;
        eprintln!("[tails] TypeScript definitions: {}", dts_path.display());
    }

    // Print summary
    let target_triple = detect_target_triple();
    eprintln!("[tails] Target: {}", target_triple);
    eprintln!(
        "[tails] Platform: {} ({})",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    Ok(())
}

pub fn run_clean() -> Result<()> {
    let manifest_dir = find_manifest_dir()?;
    let dist_dir = manifest_dir.join("dist");

    if dist_dir.exists() {
        fs::remove_dir_all(&dist_dir)?;
        eprintln!("[tails] Cleaned dist/");
    } else {
        eprintln!("[tails] dist/ already clean");
    }
    Ok(())
}

fn find_manifest_dir() -> Result<PathBuf> {
    let mut dir = std::env::current_dir().context("Failed to get current directory")?;
    loop {
        if dir.join("Cargo.toml").exists() {
            return Ok(dir);
        }
        dir = dir
            .parent()
            .context("Could not find Cargo.toml in any parent directory")?
            .to_path_buf();
    }
}

fn find_cdylib_package(cargo_toml: &str, workspace_root: &Path) -> Result<String> {
    // Parse workspace members
    let mut members = Vec::new();
    let mut in_members = false;

    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[workspace]") {
            in_members = true;
            continue;
        }
        if in_members && trimmed.starts_with('[') {
            in_members = false;
        }
        if in_members && trimmed.starts_with('"') {
            let member = trimmed.trim_matches('"').trim_matches(',');
            members.push(member.to_string());
        }
    }

    // Look for cdylib in each member
    for member in &members {
        let member_path = workspace_root.join(member).join("Cargo.toml");
        if member_path.exists() {
            let member_toml = fs::read_to_string(&member_path).unwrap_or_default();
            if member_toml.contains("cdylib") {
                // Extract package name
                for line in member_toml.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("name") && trimmed.contains('=') {
                        let name = trimmed
                            .split('=')
                            .nth(1)
                            .unwrap_or("")
                            .trim()
                            .trim_matches('"');
                        return Ok(name.to_string());
                    }
                }
            }
        }
    }

    anyhow::bail!("No cdylib package found. Specify with --package <name>")
}

fn generate_dts(lib_path: &Path, package_name: &str) -> Result<String> {
    // First, find all __TAILS_DTS_* symbols using nm
    let dts_symbols = find_all_dts_symbols(lib_path)?;

    if dts_symbols.is_empty() {
        return Ok(format!(
            "// Auto-generated TypeScript definitions for {}\n\
             // No metadata symbols found in the library.\n\
             // Add #[used] static __TAILS_DTS_* strings to your Rust module.\n",
            package_name
        ));
    }

    // Load the library to read the symbol values
    let abs_path = lib_path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {}", lib_path.display()))?;

    unsafe {
        let lib = libloading::Library::new(&abs_path)
            .with_context(|| format!("Failed to load library at: {}", abs_path.display()))?;

        let mut dts_entries: Vec<String> = Vec::new();

        for symbol_name in &dts_symbols {
            let c_name = format!("{}\0", symbol_name);
            if let Ok(func) = lib.get::<*const &str>(c_name.as_bytes()) {
                let s: &str = &**func;
                if !s.is_empty() {
                    dts_entries.push(s.to_string());
                }
            }
        }

        // Sort and deduplicate
        dts_entries.sort();
        dts_entries.dedup();

        // Parse entries and group into classes vs standalone functions
        let mut standalone_functions: Vec<String> = Vec::new();
        let mut classes: HashMap<String, ClassInfo> = HashMap::new();

        // First pass: find constructors
        for entry in &dts_entries {
            if let Some((class_name, ctor_sig)) = parse_class_constructor(entry) {
                classes
                    .entry(class_name.clone())
                    .or_insert_with(|| ClassInfo::new(&class_name))
                    .constructor = Some(ctor_sig);
            }
        }

        // Second pass: find methods and filter out class-related entries
        let class_names: Vec<String> = classes.keys().cloned().collect();
        for entry in &dts_entries {
            // Skip if already captured as a constructor
            if classes
                .values()
                .any(|c| c.constructor.as_deref() == Some(entry))
            {
                continue;
            }
            if let Some((class_name, method_sig)) = parse_class_method(entry, &class_names) {
                classes
                    .entry(class_name.clone())
                    .or_insert_with(|| ClassInfo::new(&class_name))
                    .methods
                    .push(method_sig);
            } else {
                standalone_functions.push(entry.clone());
            }
        }

        // Generate output
        let mut dts = format!(
            "// Auto-generated TypeScript definitions for {}\n\n",
            package_name
        );

        // Output classes first
        let mut class_names: Vec<&String> = classes.keys().collect();
        class_names.sort();
        for class_name in class_names {
            let class_info = &classes[class_name];
            dts.push_str(&class_info.to_dts());
            dts.push('\n');
        }

        // Output standalone functions
        for func in &standalone_functions {
            dts.push_str(func);
            dts.push('\n');
        }

        Ok(dts)
    }
}

struct ClassInfo {
    name: String,
    constructor: Option<String>,
    methods: Vec<String>,
}

impl ClassInfo {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            constructor: None,
            methods: Vec::new(),
        }
    }

    fn to_dts(&self) -> String {
        let mut output = format!("export class {} {{\n", self.name);

        if let Some(ctor) = &self.constructor {
            // Extract params from "export function ClassName(params): Type;"
            if let Some(rest) = ctor.strip_prefix("export function ") {
                if let Some(paren_start) = rest.find('(') {
                    if let Some(paren_end) = rest[paren_start..].find(')') {
                        let params = &rest[paren_start + 1..paren_start + paren_end];
                        output.push_str(&format!("  constructor({});\n", params));
                    }
                }
            }
        }

        let mut sorted_methods = self.methods.clone();
        sorted_methods.sort();
        for method in &sorted_methods {
            // Convert "export function className_methodName(params): Type;" to "  methodName(params): Type;"
            if let Some(rest) = method.strip_prefix("export function ") {
                if let Some(underscore_pos) = rest.find('_') {
                    let after_underscore = &rest[underscore_pos + 1..];
                    // Remove trailing semicolon and add proper formatting
                    let sig = after_underscore.trim_end_matches(';').trim();
                    output.push_str(&format!("  {};\n", sig));
                }
            }
        }

        output.push_str("}\n");
        output
    }
}

fn parse_class_constructor(entry: &str) -> Option<(String, String)> {
    // Pattern: "export function ClassName(params): ClassName;"
    let rest = entry.strip_prefix("export function ")?;
    let paren_pos = rest.find('(')?;
    let name = &rest[..paren_pos];

    // Check if return type matches the class name (constructor pattern)
    if let Some(colon_pos) = rest.rfind("): ") {
        let ret_type = &rest[colon_pos + 3..].trim_end_matches(';').trim();
        if *ret_type == name {
            return Some((name.to_string(), entry.to_string()));
        }
    }
    None
}

fn parse_class_method(entry: &str, class_names: &[String]) -> Option<(String, String)> {
    // Pattern: "export function className_methodName(params): Type;"
    let rest = entry.strip_prefix("export function ")?;
    let paren_pos = rest.find('(')?;
    let full_name = &rest[..paren_pos];

    // Check if this matches any known class (case-insensitive prefix)
    for class_name in class_names {
        let prefix = class_name.to_lowercase();
        if full_name.starts_with(&format!("{}_", prefix)) {
            let method_part = &full_name[prefix.len() + 1..];
            // Verify the first character is lowercase (method pattern)
            if method_part
                .chars()
                .next()
                .map(|c| c.is_lowercase())
                .unwrap_or(false)
            {
                return Some((class_name.clone(), entry.to_string()));
            }
        }
    }
    None
}

fn find_all_dts_symbols(lib_path: &Path) -> Result<Vec<String>> {
    // Use nm to find all `__TAILS_DTS_*` (legacy, unscoped) and
    // `__TAILS_<MODULE>_DTS_*` (new, module-scoped) symbols.
    let output = Command::new("nm")
        .arg("-D")
        .arg(lib_path)
        .output()
        .context("Failed to run nm. Is binutils installed?")?;

    if !output.status.success() {
        anyhow::bail!("nm failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let mut symbols = Vec::new();

    for line in stdout.lines() {
        // Format: "0000000000051d48 D __TAILS_DTS_ADD" or
        //         "0000000000051d48 D __TAILS_TAILS_FS_DTS_READ_FILE"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let sym = parts[2];
            if sym.starts_with("__TAILS_") && sym.contains("_DTS_") {
                symbols.push(sym.to_string());
            }
        }
    }

    symbols.sort();
    Ok(symbols)
}

fn extract_package_name(toml_content: &str) -> Option<String> {
    let mut in_package = false;
    for line in toml_content.lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            in_package = true;
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_package = false;
            continue;
        }
        if in_package && trimmed.starts_with("name") {
            if let Some(eq_pos) = trimmed.find('=') {
                let value = trimmed[eq_pos + 1..].trim();
                let name = value.trim_matches('"').trim_matches('\'');
                return Some(name.to_string());
            }
        }
    }
    None
}
