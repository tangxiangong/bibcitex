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
        _ => Err(
            "Usage: xtask release-metadata | verify-release <directory> | publish-release".into(),
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
