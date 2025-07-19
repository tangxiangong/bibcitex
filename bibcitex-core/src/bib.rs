use crate::Result;
use biblatex::Bibliography;
use fs_err as fs;
use std::path::Path;

/// Parse BibTeX database `.bib` file
pub fn parse(file_path: impl AsRef<Path>) -> Result<Bibliography> {
    let src = fs::read_to_string(file_path)?;
    Ok(Bibliography::parse(&src)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse() {
        let path = Path::new("./database.bib");
        let bib = parse(path).unwrap();
        let entry = bib.get("MR4293957").unwrap();
        let title = entry.title().unwrap();
        println!("{title:#?}");
    }
}
