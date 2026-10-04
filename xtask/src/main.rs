use std::{env, fs::OpenOptions, io::Write, path::Path, process::ExitCode};
use xtask::{Result, github, publish_release, verify_release, versions};

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["release-metadata"] => {
            let version = versions(&env::var("RELEASE_TAG")?)?;
            let mut output = OpenOptions::new().append(true).create(true).open(env::var("GITHUB_OUTPUT")?)?;
            writeln!(output, "version={}\nwindows_version={}", version.version, version.windows)?;
            Ok(())
        }
        ["verify-release", platform, directory, version, repository, tag] => verify_release(platform, Path::new(directory), version, repository, tag),
        ["legacy-sign", path] => xtask::legacy::sign(Path::new(path)),
        ["legacy-manifest", directory, repository, tag] => xtask::legacy::write_manifest(Path::new(directory), repository, tag),
        ["publish-release"] => publish_release(&env::var("RELEASE_TAG")?, Path::new(&env::var("RELEASE_ASSETS")?), &env::var("GITHUB_REPOSITORY")?, github),
        _ => Err("Usage: xtask release-metadata | verify-release <platform> <directory> <version> <repository> <tag> | legacy-sign <path> | legacy-manifest <directory> <repository> <tag> | publish-release".into()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
