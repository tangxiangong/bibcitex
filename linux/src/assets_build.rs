//! Stage the shared artwork inside the NSH bundle boundary before describing the crate.
use std::{env, fs, io, path::Path};

mod locale_build;
mod locale_key;

fn copy_asset(source: &Path, target: &Path) -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-changed={}", target.display());
    let contents = fs::read(source)?;
    if fs::read(target).ok().as_ref() != Some(&contents) {
        fs::write(target, contents)?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    // NSH may put its generated Cargo manifest inside .nsh/rust/app-crates.
    let manifest = env::var_os("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR");
    let app = Path::new(&manifest)
        .ancestors()
        .find(|p| p.join("nsh.json").is_file())
        .expect("NSH client root");
    locale_build::generate(app)?;
    let public = app.join("../public");
    let staged = app.join(".nsh/resources");
    fs::create_dir_all(&staged)?;
    let config_path = app.join("nsh.json");
    println!("cargo:rerun-if-changed={}", config_path.display());
    let config: serde_json::Value = serde_json::from_slice(&fs::read(config_path)?)?;
    let channels = config["updater"]["channels"]
        .as_object()
        .map(|v| v.keys().collect::<Vec<_>>())
        .unwrap_or_default();
    let generated = app.join(".nsh/generated");
    fs::create_dir_all(&generated)?;
    let config_ts = format!(
        "// Generated from nsh.json by assets_build.rs.\nexport const appVersion = {};\nexport const updateChannels: readonly string[] = {};\n",
        config["version"],
        serde_json::to_string(&channels)?
    );
    let config_target = generated.join("config.ts");
    println!("cargo:rerun-if-changed={}", config_target.display());
    if fs::read_to_string(&config_target).ok().as_ref() != Some(&config_ts) {
        fs::write(config_target, config_ts)?;
    }
    copy_asset(&public.join("favicon.png"), &staged.join("favicon.png"))?;
    copy_asset(&public.join("logo.png"), &staged.join("app-icon.png"))?;
    for icons in ["icons", "icons/windows"] {
        let source = public.join(icons);
        let target = staged.join(icons);
        fs::create_dir_all(&target)?;
        println!("cargo:rerun-if-changed={}", source.display());
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "svg") || entry.file_name() == "LICENSE" {
                copy_asset(&path, &target.join(entry.file_name()))?;
            }
        }
    }
    Ok(())
}
