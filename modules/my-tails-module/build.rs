use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // Determine the library filename based on platform
    let lib_name = if cfg!(target_os = "windows") {
        "my_tails_module.dll"
    } else if cfg!(target_os = "macos") {
        "libmy_tails_module.dylib"
    } else {
        "libmy_tails_module.so"
    };

    // The built library will be in target/<profile>/
    let src_path = PathBuf::from(&target_dir).join(&profile).join(lib_name);

    // Output to ./dist/
    let dist_dir = std::path::Path::new("dist");
    fs::create_dir_all(dist_dir).unwrap();

    let dst_path = dist_dir.join(lib_name);

    // Copy the built library to dist/
    if src_path.exists() {
        fs::copy(&src_path, &dst_path).expect("Failed to copy library to dist/");
        println!("cargo:warning=Copied {} to dist/", lib_name);
    } else {
        println!(
            "cargo:warning=Library not found at {}. Run cargo build --release first.",
            src_path.display()
        );
    }
}
