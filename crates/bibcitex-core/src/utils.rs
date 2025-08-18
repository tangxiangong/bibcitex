use crate::bib::Reference;
use biblatex::{Bibliography, Chunk, Chunks};
use rayon::prelude::*;

/// Read a bibliography and convert it to a vector of references parallelly.
pub fn par_read_bibliography(bibliography: Bibliography) -> Vec<Reference> {
    bibliography
        .into_vec()
        .par_iter()
        .map(Reference::from)
        .collect()
}

/// Read a bibliography and convert it to a vector of references serially.
pub fn serial_read_bibliography(bibliography: Bibliography) -> Vec<Reference> {
    bibliography
        .into_vec()
        .iter()
        .map(Reference::from)
        .collect()
}

/// Read a bibliography and convert it to a vector of references.
///
/// # Note
///
/// If the length of the bibliography is greater than 80, it will use parallel reading.
pub fn read_bibliography(bibliography: Bibliography) -> Vec<Reference> {
    if bibliography.len() > 80 {
        par_read_bibliography(bibliography)
    } else {
        serial_read_bibliography(bibliography)
    }
}

/// Merge [`Chunk::Normal`] and [`Chunk::Verbatim`] chunks.
pub fn merge_chunks(chunks: Chunks) -> Vec<Chunk> {
    let mut chunks_iter = chunks.into_iter();
    let mut merged: Vec<Chunk> = Vec::with_capacity(chunks_iter.len());
    if let Some(first_chunk) = chunks_iter.next() {
        let mut current = first_chunk.v;
        for chunk in chunks_iter {
            match (&current, &chunk.v) {
                (
                    Chunk::Normal(_) | Chunk::Verbatim(_),
                    Chunk::Normal(txt) | Chunk::Verbatim(txt),
                ) => current.get_mut().push_str(txt),
                _ => {
                    merged.push(current);
                    current = chunk.v;
                }
            }
        }
        merged.push(current);
    }
    merged
}

/// Abbreviate a path string to a maximum length.
pub fn abbr_path(path_str: &str, max_length: usize) -> String {
    if path_str.len() <= max_length {
        path_str.to_string()
    } else {
        _abbr_path(path_str, max_length)
    }
}

/// Abbreviate a path string to a maximum length.
fn _abbr_path(path_str: &str, max_length: usize) -> String {
    let parts: Vec<&str> = path_str
        .split(std::path::MAIN_SEPARATOR)
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() <= 2 {
        return path_str.to_string();
    }

    let separator = std::path::MAIN_SEPARATOR.to_string();
    let ellipsis = "...";
    let is_absolute = path_str.starts_with(std::path::MAIN_SEPARATOR);

    for prefix_len in 1..parts.len() {
        for suffix_len in 1..(parts.len() - prefix_len) {
            let prefix_parts = &parts[..prefix_len];
            let suffix_parts = &parts[parts.len() - suffix_len..];

            let abbreviated = if is_absolute {
                format!(
                    "{}{}{}{}{}{}",
                    separator,
                    prefix_parts.join(&separator),
                    separator,
                    ellipsis,
                    separator,
                    suffix_parts.join(&separator)
                )
            } else {
                format!(
                    "{}{}{}{}{}",
                    prefix_parts.join(&separator),
                    separator,
                    ellipsis,
                    separator,
                    suffix_parts.join(&separator)
                )
            };

            if abbreviated.len() <= max_length {
                return abbreviated;
            }
        }
    }

    let first_part = parts[0];
    let last_part = parts[parts.len() - 1];
    if is_absolute {
        format!("{separator}{first_part}{separator}{ellipsis}{separator}{last_part}")
    } else {
        format!("{first_part}{separator}{ellipsis}{separator}{last_part}")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bib::parse;
    use std::path::Path;

    #[test]
    fn test_read_bibliography() {
        let path = Path::new("../database.bib");
        let bib = parse(path).unwrap();
        let bib_len = bib.len();
        let references = read_bibliography(bib);
        assert_eq!(references.len(), bib_len);
    }
}

#[test]
fn test_abbr_path() {
    let result = abbr_path("/Users/username/Documents/Rust/demo/src/main.rs", 35);
    println!("Result: {result}");
    assert!(result.starts_with("/Users"));
    assert!(result.ends_with("main.rs"));
    assert!(result.contains("..."));
}

#[test]
fn test_abbr_path_comprehensive() {
    // 测试不同长度限制下的缩写
    let path = "/Users/username/Documents/Rust/demo/src/main.rs";

    // 长度限制 30
    let result30 = abbr_path(path, 30);
    println!("Length 30: {result30}");
    assert!(result30.len() <= 30);

    // 长度限制 25
    let result25 = abbr_path(path, 25);
    println!("Length 25: {result25}");
    assert!(result25.len() <= 25);

    // 长度限制 20
    let result20 = abbr_path(path, 20);
    println!("Length 20: {result20}");
    assert!(result20.len() <= 20);

    // 测试相对路径
    let rel_path = "src/components/ui/button.tsx";
    let rel_result = abbr_path(rel_path, 20);
    println!("Relative: {rel_result}");
    assert!(rel_result.len() <= 20);
    assert!(!rel_result.starts_with("/"));
}
