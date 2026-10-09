//! Release-note selection happens before packaging, identically for every client.
use crate::{Result, ensure};
use std::{fs, path::Path};

const PLATFORMS: [&str; 3] = ["macos", "windows", "linux"];

/// Select common and matching platform blocks, rejecting ambiguous directives.
/// Directives occupy their own line; nesting is intentionally unsupported.
pub fn select(markdown: &str, platform: &str) -> Result<String> {
    ensure(PLATFORMS.contains(&platform), "Unknown target platform")?;
    render(markdown, Some(platform))
}

/// Render all platforms for GitHub, with visible platform labels.
pub fn github(markdown: &str) -> Result<String> {
    render(markdown, None)
}

fn render(markdown: &str, platform: Option<&str>) -> Result<String> {
    let mut selected = None;
    let mut output = String::new();
    for line in markdown.split_inclusive('\n') {
        let directive = line.trim();
        if let Some(targets) = directive
            .strip_prefix("<!-- platform:")
            .and_then(|value| value.strip_suffix(" -->"))
        {
            ensure(selected.is_none(), "Nested platform blocks are forbidden")?;
            let targets: Vec<_> = targets.split(',').map(str::trim).collect();
            ensure(
                targets
                    .iter()
                    .all(|target| *target == "all" || PLATFORMS.contains(target)),
                "Unknown or empty platform selector",
            )?;
            ensure(
                !targets.contains(&"all") || targets.len() == 1,
                "Use all alone",
            )?;
            selected =
                Some(platform.is_none_or(|platform| {
                    targets.contains(&"all") || targets.contains(&platform)
                }));
            if platform.is_none() {
                let labels: Vec<_> = targets
                    .iter()
                    .map(|target| match *target {
                        "macos" => "macOS",
                        "windows" => "Windows",
                        "all" => "All platforms",
                        _ => "Linux",
                    })
                    .collect();
                output.push_str(&format!("\n**{}**\n\n", labels.join(" / ")));
            }
        } else if directive == "<!-- /platform -->" {
            ensure(
                selected.take().is_some(),
                "Unmatched platform closing marker",
            )?;
            if platform.is_none() {
                output.push_str("\n---\n\n");
            }
        } else {
            ensure(
                !line.split("<!--").skip(1).any(|comment| {
                    let comment = comment.trim_start().to_ascii_lowercase();
                    comment.starts_with("platform") || comment.starts_with("/platform")
                }),
                "Platform directives must use the documented syntax on a separate line",
            )?;
            if selected.unwrap_or(true) {
                output.push_str(line);
            }
        }
    }
    ensure(selected.is_none(), "Unclosed platform block")?;
    Ok(output)
}

/// Validate both locales and generate platform assets plus Velopack's bilingual input.
pub fn generate(source: &Path, destination: &Path) -> Result<()> {
    let mut files = Vec::new();
    let mut windows = String::new();
    let mut body = String::new();
    for locale in ["zh-Hans", "en"] {
        let markdown = fs::read_to_string(source.join(format!("{locale}.md")))?;
        let language = if locale == "en" {
            "English"
        } else {
            "简体中文"
        };
        body.push_str(&format!("## {language}\n\n{}\n\n", github(&markdown)?));
        for platform in PLATFORMS {
            let text = select(&markdown, platform)?;
            ensure(
                !text.trim().is_empty(),
                "Release notes must not be empty for any platform",
            )?;
            if platform == "macos" {
                // Keep existing Sparkle links and mirror compatibility.
                files.push((format!("notes-{locale}.md"), text.clone()));
            }
            if platform == "windows" {
                windows.push_str(&format!(
                    "<!-- locale:{locale} -->\n{text}\n<!-- /locale -->\n"
                ));
            }
            files.push((format!("notes-{platform}-{locale}.md"), text));
        }
    }
    files.push(("notes-windows.md".into(), windows));
    files.push(("release-body.md".into(), body));
    fs::create_dir_all(destination)?;
    for (name, text) in files {
        fs::write(destination.join(name), text)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_and_multi_platform_content_keep_source_order() {
        let source = "common\n<!-- platform:macos,linux -->\nunix\n<!-- /platform -->\n<!-- platform:windows -->\nwin\n<!-- /platform -->\n<!-- platform:all -->\nall\n<!-- /platform -->\nend";
        assert_eq!(select(source, "macos").unwrap(), "common\nunix\nall\nend");
        assert_eq!(select(source, "linux").unwrap(), "common\nunix\nall\nend");
        assert_eq!(select(source, "windows").unwrap(), "common\nwin\nall\nend");
        assert_eq!(select("legacy\r\n", "windows").unwrap(), "legacy\r\n");
    }

    #[test]
    fn invalid_directives_fail_even_in_hidden_blocks() {
        for source in [
            "<!-- platform:android -->",
            "<!--platform:windows-->",
            "<!--  platform:windows -->",
            "<!-- Platform:windows -->",
            "<!--\t/platform -->",
            "<!-- platform: -->",
            "<!-- platform:all,macos -->",
            "<!-- /platform -->",
            "<!-- platform:macos -->",
            "text <!-- platform:macos -->",
            "<!-- platform:windows -->\n<!-- platform:macos -->\n<!-- /platform -->\n<!-- /platform -->",
        ] {
            assert!(select(source, "linux").is_err(), "{source}");
            assert!(github(source).is_err(), "{source}");
        }
        assert!(select("common", "android").is_err());
    }

    #[test]
    fn github_includes_every_platform_with_labels() {
        let source = "common\n<!-- platform:macos,windows -->\ndesktop\n<!-- /platform -->\n<!-- platform:linux -->\nlinux\n<!-- /platform -->\n<!-- platform:all -->\nshared\n<!-- /platform -->\n";
        assert_eq!(
            github(source).unwrap(),
            "common\n\n**macOS / Windows**\n\ndesktop\n\n---\n\n\n**Linux**\n\nlinux\n\n---\n\n\n**All platforms**\n\nshared\n\n---\n\n"
        );
    }

    #[test]
    fn invalid_second_locale_writes_no_partial_assets() {
        let source = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::write(source.path().join("zh-Hans.md"), "通用更新").unwrap();
        fs::write(source.path().join("en.md"), "<!-- platform:macos -->").unwrap();
        assert!(generate(source.path(), output.path()).is_err());
        assert_eq!(fs::read_dir(output.path()).unwrap().count(), 0);
    }

    #[test]
    fn whitespace_and_crlf_are_supported() {
        assert_eq!(
            select(
                "  <!-- platform:macos, linux -->\r\nkeep\r\n  <!-- /platform -->\r\n",
                "linux"
            )
            .unwrap(),
            "keep\r\n"
        );
    }

    #[test]
    fn generated_assets_select_platform_before_locale_packaging() {
        let source = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        for locale in ["zh-Hans", "en"] {
            fs::write(
                source.path().join(format!("{locale}.md")),
                format!("{locale}\n<!-- platform:windows -->\nwindows only\n<!-- /platform -->\n"),
            )
            .unwrap();
        }
        generate(source.path(), output.path()).unwrap();
        assert_eq!(
            fs::read_to_string(output.path().join("release-body.md")).unwrap(),
            "## 简体中文\n\nzh-Hans\n\n**Windows**\n\nwindows only\n\n---\n\n\n\n## English\n\nen\n\n**Windows**\n\nwindows only\n\n---\n\n\n\n"
        );
        assert_eq!(
            fs::read_to_string(output.path().join("notes-en.md")).unwrap(),
            "en\n"
        );
        assert_eq!(
            fs::read_to_string(output.path().join("notes-linux-en.md")).unwrap(),
            "en\n"
        );
        assert_eq!(
            fs::read_to_string(output.path().join("notes-windows.md")).unwrap(),
            "<!-- locale:zh-Hans -->\nzh-Hans\nwindows only\n\n<!-- /locale -->\n<!-- locale:en -->\nen\nwindows only\n\n<!-- /locale -->\n"
        );
    }
}
