use std::{env, fs::OpenOptions, io::Write, path::Path, process::ExitCode};
use xtask::{Result, github, publish_release, versions};
fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["release-notes", source, destination] => xtask::notes::generate(Path::new(source), Path::new(destination)),
        ["package-linux", binary, directory, arch] => xtask::linux::package(Path::new(binary), Path::new(directory), arch),
        ["sign-linux", directory, arch] => xtask::linux::sign(Path::new(directory), arch),
        ["release-metadata"] => {
            let version = versions(&env::var("RELEASE_TAG")?)?;
            if version.version != env!("CARGO_PKG_VERSION") {
                return Err("Release tag does not match the checked-out workspace version".into());
            }
            let mut output = OpenOptions::new()
                .append(true)
                .create(true)
                .open(env::var("GITHUB_OUTPUT")?)?;
            writeln!(
                output,
                "version={}\nchannel={}",
                version.version, version.channel
            )?;
            Ok(())
        }
        ["verify-release", directory] => xtask::prepare(
            Path::new(directory),
            &env::var("RELEASE_TAG")?,
            &env::var("GITHUB_REPOSITORY")?,
            &env::var("SPARKLE_PUBLIC_ED_KEY")?,
        )
        .map(|_| ()),
        ["publish-release"] => publish_release(
            &env::var("RELEASE_TAG")?,
            Path::new(&env::var("RELEASE_ASSETS")?),
            &env::var("GITHUB_REPOSITORY")?,
            &env::var("SPARKLE_PUBLIC_ED_KEY")?,
            github,
        ),
        ["publish-cos"] => xtask::cos::publish(
            &env::var("RELEASE_TAG")?,
            Path::new(&env::var("RELEASE_ASSETS")?),
            &env::var("GITHUB_REPOSITORY")?,
            &env::var("SPARKLE_PUBLIC_ED_KEY")?,
        ),
        _ => Err(
            "Usage: xtask release-notes <source> <destination> | release-metadata | verify-release <directory> | publish-release | publish-cos | package-linux <binary> <directory> <arch> | sign-linux <directory> <arch>".into(),
        ),
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
