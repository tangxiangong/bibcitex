//! Compile the root catalogs without interpreting punctuation as key hierarchy.
use std::{collections::BTreeMap, fs, io, path::Path};

use crate::locale_key::key;

pub(crate) fn template(source: &str) -> String {
    let mut result = String::new();
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '%' => result.push_str("%{pct}"),
            '{' => {
                let mut digits = String::new();
                while let Some(digit) = chars.peek().copied().filter(char::is_ascii_digit) {
                    chars.next();
                    digits.push(digit);
                }
                if !digits.is_empty() && chars.peek() == Some(&'}') {
                    chars.next();
                    result.push_str(&format!("%{{p{digits}}}"));
                } else {
                    result.push('{');
                    result.push_str(&digits);
                }
            }
            _ => result.push(ch),
        }
    }
    result
}

pub(crate) fn generate(app: &Path) -> io::Result<()> {
    let target = app.join("locales");
    fs::create_dir_all(&target)?;
    println!("cargo:rerun-if-changed={}", target.display());
    for locale in ["en", "zh-Hans"] {
        let source = app.join(format!("../localization/{locale}.json"));
        let output = target.join(format!("{locale}.json"));
        println!("cargo:rerun-if-changed={}", source.display());
        println!("cargo:rerun-if-changed={}", output.display());
        let catalog: BTreeMap<String, String> = serde_json::from_slice(&fs::read(source)?)?;
        let compiled: BTreeMap<_, _> = catalog.iter().map(|(k, v)| (key(k), template(v))).collect();
        if compiled.len() != catalog.len() {
            return Err(io::Error::other("Locale key collision"));
        }
        let bytes = serde_json::to_vec_pretty(&compiled)?;
        if fs::read(&output).ok().as_ref() != Some(&bytes) {
            fs::write(output, bytes)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nsh_core::i18n::Translations;

    #[test]
    fn every_root_entry_survives_the_actual_nsh_loader_and_interpolator() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app");
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../localization");
        fs::create_dir_all(&app).unwrap();
        fs::create_dir_all(dir.path().join("localization")).unwrap();
        for lang in ["en", "zh-Hans"] {
            fs::copy(
                root.join(format!("{lang}.json")),
                dir.path().join(format!("localization/{lang}.json")),
            )
            .unwrap();
        }
        generate(&app).unwrap();
        let loaded = nsh_i18n::read_translations(&app).unwrap();
        let translations = Translations::from_json(&nsh_i18n::translations_json(&loaded)).unwrap();
        for lang in ["en", "zh-Hans"] {
            let catalog: BTreeMap<String, String> =
                serde_json::from_slice(&fs::read(root.join(format!("{lang}.json"))).unwrap())
                    .unwrap();
            assert_eq!(loaded[lang].len(), catalog.len(), "{lang}");
            for (original, text) in catalog {
                let encoded = key(&original);
                let units = encoded.as_bytes()[1..]
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|part| {
                        u16::from_str_radix(std::str::from_utf8(part).unwrap(), 16).unwrap()
                    })
                    .collect::<Vec<_>>();
                assert_eq!(String::from_utf16(&units).unwrap(), original);
                assert_eq!(
                    translations.translate(lang, &encoded, &[("pct", "%".into())]),
                    restore_placeholders(&text),
                    "{lang}: {original}"
                );
                let values = (0..20)
                    .map(|i| (format!("p{i}"), format!("arg{i}%{{p0}}")))
                    .collect::<Vec<_>>();
                let mut args = values
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.clone()))
                    .collect::<Vec<_>>();
                args.push(("pct", "%".into()));
                let expected = (0..20).fold(text.clone(), |s, i| {
                    s.replace(&format!("{{{i}}}"), &format!("arg{i}%{{p0}}"))
                });
                assert_eq!(
                    translations.translate(lang, &encoded, &args),
                    expected,
                    "{lang}: {original}"
                );
            }
        }
    }

    fn restore_placeholders(text: &str) -> String {
        // Unfilled named placeholders stay in NSH's template form.
        template(text).replace("%{pct}", "%")
    }

    #[test]
    fn punctuation_astral_keys_and_literal_percent_patterns_are_lossless() {
        let source = "100% %{p0} % {0} {1} {x} 🦀";
        let converted = template(source);
        let translations = Translations::from_json(
            &serde_json::json!({"en": {key("a.b: (🦀)"): converted}}).to_string(),
        )
        .unwrap();
        assert_eq!(
            translations.translate(
                "en",
                &key("a.b: (🦀)"),
                &[
                    ("pct", "%".into()),
                    ("p0", "A%{p1}".into()),
                    ("p1", "B".into())
                ]
            ),
            "100% %{p0} % A%{p1} B {x} 🦀"
        );
    }

    #[test]
    fn digit_index_boundaries_and_incomplete_placeholders_survive_interpolation() {
        let source = "{0}|{9}|{10}|{99}|{100}|{x}|{1x}|{}|{-1}|{١}";
        let translations = Translations::from_json(
            &serde_json::json!({"en": {key("indices"): template(source)}}).to_string(),
        )
        .unwrap();
        let values = [0, 9, 10, 99, 100]
            .map(|index| (format!("p{index}"), format!("v{index}")))
            .into_iter()
            .collect::<Vec<_>>();
        let args = values
            .iter()
            .map(|(key, value)| (key.as_str(), value.clone()))
            .collect::<Vec<_>>();

        assert_eq!(
            translations.translate("en", &key("indices"), &args),
            "v0|v9|v10|v99|v100|{x}|{1x}|{}|{-1}|{١}"
        );
    }
}
