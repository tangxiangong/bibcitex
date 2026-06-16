fn main() {
    tauri_build::build();

    #[cfg(target_os = "macos")]
    compile_native_helper();
}

#[cfg(target_os = "macos")]
fn compile_native_helper() {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let helper_dir = manifest_dir.join("..").join("macos").join("NativeHelper");
    let package_dir = helper_dir.join("Package.swift");
    println!("cargo:rerun-if-changed={}", package_dir.display());
    println!(
        "cargo:rerun-if-changed={}",
        helper_dir.join("Sources/NativeHelper").display()
    );

    let skip_on_failure = env::var("BIBCITEX_SKIP_NATIVE_HELPER_BUILD")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "True"))
        .unwrap_or(false);

    fn rerun_if_changed_dir(path: &Path) {
        let mut dirs = vec![path.to_path_buf()];
        while let Some(next) = dirs.pop() {
            let Ok(entries) = fs::read_dir(&next) else {
                continue;
            };
            for entry in entries.flatten() {
                let child = entry.path();
                if child.is_dir() {
                    dirs.push(child);
                } else {
                    println!("cargo:rerun-if-changed={}", child.display());
                }
            }
        }
    }
    rerun_if_changed_dir(&helper_dir.join("Sources"));

    let status = Command::new("swift")
        .arg("build")
        .arg("--package-path")
        .arg(&helper_dir)
        .arg("--configuration")
        .arg("release")
        .arg("--product")
        .arg("NativeHelper")
        .status()
        .expect("failed to execute swift build");

    if !status.success() {
        if skip_on_failure {
            println!(
                "cargo:warning=swift build for NativeHelper failed, skipping native helper link step"
            );
            return;
        }
        panic!("swift build for NativeHelper failed");
    }

    let bin_path_output = Command::new("swift")
        .arg("build")
        .arg("--package-path")
        .arg(&helper_dir)
        .arg("--configuration")
        .arg("release")
        .arg("--product")
        .arg("NativeHelper")
        .arg("--show-bin-path")
        .output()
        .expect("failed to query swift build bin path");

    if !bin_path_output.status.success() {
        if skip_on_failure {
            println!(
                "cargo:warning=failed to get swift binary output path, skipping native helper link step"
            );
            return;
        }
        panic!("failed to query swift binary output path");
    }

    let output_dir = String::from_utf8_lossy(&bin_path_output.stdout)
        .trim()
        .to_string();
    if output_dir.is_empty() {
        if skip_on_failure {
            println!(
                "cargo:warning=swift helper output path is empty, skipping native helper link step"
            );
            return;
        }
        panic!("swift helper output path is empty");
    }

    let output_dir = PathBuf::from(output_dir);
    let static_lib = output_dir.join("libNativeHelper.a");
    let dylib = output_dir.join("libNativeHelper.dylib");

    if static_lib.exists() {
        println!("cargo:rustc-link-lib=static=NativeHelper");
    } else if dylib.exists() {
        println!("cargo:rustc-link-lib=dylib=NativeHelper");
    } else if !skip_on_failure {
        panic!("NativeHelper binary not found in {}", output_dir.display());
    } else {
        println!(
            "cargo:warning=NativeHelper binary not found in {}, skipping native helper link step",
            output_dir.display()
        );
        return;
    }

    println!("cargo:rustc-link-search=native={}", output_dir.display());
}
