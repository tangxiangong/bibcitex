//! Citation source text shared by copying and cross-application pasting.

/// Stable preference values and source-language labels for the format picker.
pub const CITATION_FORMATS: &[(&str, &str)] = &[
    ("raw", "key"),
    ("cite", "\\cite"),
    ("citep", "\\citep (natbib)"),
    ("citet", "\\citet (natbib)"),
    ("autocite", "\\autocite (biblatex)"),
    ("parencite", "\\parencite (biblatex)"),
    ("textcite", "\\textcite (biblatex)"),
    ("footcite", "\\footcite (biblatex)"),
    ("typst", "@key"),
    ("typst-cite", "#cite"),
    ("typst-prose", "#cite (正文引用)"),
];

/// Format keys already ordered by the result list and selected by record UUID.
/// Raw keys always use commas; individual commands are separated by spaces.
/// Equal keys from distinct records are preserved. Unknown preferences fall back
/// to raw keys and merged output, so older/newer clients remain usable.
pub fn format_citation(keys: &[String], format: &str, mode: &str) -> String {
    let individual = mode == "individual";
    if keys.is_empty() {
        return String::new();
    }
    match format {
        "cite" | "citep" | "citet" | "autocite" | "parencite" | "textcite" | "footcite" => {
            if individual {
                keys.iter()
                    .map(|key| format!("\\{format}{{{key}}}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                format!("\\{format}{{{}}}", keys.join(","))
            }
        }
        "typst" | "typst-cite" | "typst-prose" => keys
            .iter()
            .map(|key| typst_citation(key, format))
            .collect::<Vec<_>>()
            // A source newline is whitespace and still groups adjacent cites.
            // An empty inline box separates the groups without visible content.
            .join(if individual { " #box[] " } else { " " }),
        _ => keys.join(","),
    }
}

fn typst_citation(key: &str, format: &str) -> String {
    if format == "typst"
        && key.starts_with(|c: char| c.is_alphanumeric() || c == '_')
        && key.ends_with(|c: char| c.is_alphanumeric() || c == '_')
        && key
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | ':' | '.'))
    {
        return format!("@{key}");
    }
    let mut escaped = String::new();
    for c in key.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => escaped.push(c),
        }
    }
    let form = if format == "typst-prose" {
        ", form: \"prose\""
    } else {
        ""
    };
    format!("#cite(label(\"{escaped}\"){form})")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> Vec<String> {
        ["z2026", "a2025"].map(String::from).to_vec()
    }

    #[test]
    fn defaults_preserve_list_order_and_distinct_records_with_equal_keys() {
        assert_eq!(format_citation(&keys(), "raw", "merged"), "z2026,a2025");
        assert_eq!(format_citation(&keys(), "raw", "individual"), "z2026,a2025");
        assert_eq!(format_citation(&keys(), "future", "future"), "z2026,a2025");
        assert_eq!(
            format_citation(&["a".into(), "a".into()], "raw", "merged"),
            "a,a"
        );
    }

    #[test]
    fn latex_presets_support_single_merged_and_individual_commands() {
        for command in [
            "cite",
            "citep",
            "citet",
            "autocite",
            "parencite",
            "textcite",
            "footcite",
        ] {
            assert_eq!(
                format_citation(&keys(), command, "merged"),
                format!("\\{command}{{z2026,a2025}}")
            );
            assert_eq!(
                format_citation(&keys(), command, "individual"),
                format!("\\{command}{{z2026}} \\{command}{{a2025}}")
            );
            assert_eq!(
                format_citation(&keys()[..1], command, "individual"),
                format!("\\{command}{{z2026}}")
            );
        }
    }

    #[test]
    fn typst_keeps_merged_citations_adjacent_and_separates_individual_groups() {
        for (preset, first, second) in [
            ("typst", "@z2026", "@a2025"),
            (
                "typst-cite",
                "#cite(label(\"z2026\"))",
                "#cite(label(\"a2025\"))",
            ),
            (
                "typst-prose",
                "#cite(label(\"z2026\"), form: \"prose\")",
                "#cite(label(\"a2025\"), form: \"prose\")",
            ),
        ] {
            assert_eq!(
                format_citation(&keys(), preset, "merged"),
                format!("{first} {second}")
            );
            assert_eq!(
                format_citation(&keys(), preset, "individual"),
                format!("{first} #box[] {second}")
            );
            assert_eq!(format_citation(&keys()[..1], preset, "merged"), first);
        }
    }

    #[test]
    fn typst_special_keys_use_escaped_label_strings_without_changing_identity() {
        let key = "DBLP:books/引用\\\"\n\t";
        let expected = "#cite(label(\"DBLP:books/引用\\\\\\\"\\n\\t\"))";
        assert_eq!(format_citation(&[key.into()], "typst", "merged"), expected);
        assert_eq!(
            format_citation(&["key.".into()], "typst", "merged"),
            "#cite(label(\"key.\"))"
        );
        assert_eq!(
            format_citation(&["研究-2026".into()], "typst", "merged"),
            "@研究-2026"
        );
    }

    #[test]
    fn empty_selections_never_generate_empty_commands() {
        for (format, _) in CITATION_FORMATS {
            for mode in ["merged", "individual"] {
                assert!(format_citation(&[], format, mode).is_empty());
            }
        }
    }
}
