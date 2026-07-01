use anyhow::{Context, Result};
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

pub fn run_build(opts: BuildOptions) -> Result<()> {
    let manifest_dir = find_manifest_dir()?;
    let workspace_root = &manifest_dir;

    // Find Cargo.toml to get package name
    let cargo_toml = fs::read_to_string(workspace_root.join("Cargo.toml"))
        .context("Failed to read Cargo.toml")?;

    let package_name = if let Some(ref pkg) = opts.package {
        pkg.clone()
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
    use std::os::raw::c_char;

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
    // Use full absolute path to ensure libloading can find it
    let abs_path = lib_path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {}", lib_path.display()))?;

    unsafe {
        let lib = libloading::Library::new(&abs_path)
            .with_context(|| format!("Failed to load library at: {}", abs_path.display()))?;

        let mut dts_entries: Vec<String> = Vec::new();

        for symbol_name in &dts_symbols {
            let c_name = format!("{}\0", symbol_name);
            if let Ok(func) = lib.get::<*const c_char>(c_name.as_bytes()) {
                let c_str = *func;
                if !c_str.is_null() {
                    if let Ok(rust_str) = std::ffi::CStr::from_ptr(c_str).to_str() {
                        if !rust_str.is_empty() {
                            dts_entries.push(rust_str.to_string());
                        }
                    }
                }
            }
        }

        // Sort and deduplicate
        dts_entries.sort();
        dts_entries.dedup();

        let mut dts = format!(
            "// Auto-generated TypeScript definitions for {}\n\n",
            package_name
        );
        for entry in &dts_entries {
            dts.push_str(entry);
            dts.push('\n');
        }
        Ok(dts)
    }
}

fn find_all_dts_symbols(lib_path: &Path) -> Result<Vec<String>> {
    // Use nm to find all __TAILS_DTS_* symbols
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
        // Format: "0000000000051d48 D __TAILS_DTS_ADD"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let sym = parts[2];
            if sym.starts_with("__TAILS_DTS_") {
                symbols.push(sym.to_string());
            }
        }
    }

    symbols.sort();
    Ok(symbols)
}
