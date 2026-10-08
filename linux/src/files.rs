//! Resolve native paths and bibliography attachment entries without opening them.
use std::path::{Path, PathBuf};

fn entry_path(value: &str) -> Option<String> {
    let first = value.find(':')?;
    let last = value.rfind(':')?;
    let format = &value[last + 1..];
    if first == last
        || first + 1 == last
        || format.is_empty()
        || !format
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'+'))
    {
        return None;
    }
    Some(
        value[first + 1..last]
            .replace("\\;", ";")
            .replace("\\:", ":"),
    )
}

pub fn attachment_path(
    value: &str,
    bibliography: &Path,
    home: Option<&Path>,
) -> Result<PathBuf, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("Invalid bibliography path".into());
    }
    let extracted = if value.starts_with("file:") {
        value.to_owned()
    } else {
        value
            .match_indices(';')
            .filter(|(i, _)| *i == 0 || value.as_bytes()[i - 1] != b'\\')
            .find_map(|(i, _)| entry_path(&value[..i]))
            .or_else(|| entry_path(value))
            .unwrap_or_else(|| value.to_owned())
    };
    if extracted.starts_with("file:") {
        return url::Url::parse(&extracted)
            .map_err(|e| e.to_string())?
            .to_file_path()
            .map_err(|_| "Invalid bibliography path".into());
    }
    if extracted == "~" || extracted.starts_with("~/") {
        return Ok(home
            .ok_or("Home directory unavailable")?
            .join(extracted.strip_prefix("~/").unwrap_or("")));
    }
    let path = PathBuf::from(extracted);
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(bibliography
            .parent()
            .ok_or("Invalid bibliography path")?
            .join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolves_first_attachment_relative_to_its_bibliography() {
        assert_eq!(
            attachment_path(
                "paper:pdf/paper.pdf:PDF;other:other.pdf:PDF",
                Path::new("/papers/refs.bib"),
                None
            )
            .unwrap(),
            Path::new("/papers/pdf/paper.pdf")
        );
    }
    #[test]
    fn preserves_literal_filename_characters_and_decodes_only_file_uris() {
        let library = Path::new("/papers/refs.bib");
        assert_eq!(
            attachment_path("draft;50%#?.pdf", library, None).unwrap(),
            Path::new("/papers/draft;50%#?.pdf")
        );
        assert_eq!(
            attachment_path("file:///papers/a%20b.pdf", library, None).unwrap(),
            Path::new("/papers/a b.pdf")
        );
        assert_eq!(
            attachment_path("~/a.pdf", library, Some(Path::new("/users/test"))).unwrap(),
            Path::new("/users/test/a.pdf")
        );
    }
}
