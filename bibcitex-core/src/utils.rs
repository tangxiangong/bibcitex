use crate::bib::Reference;
use biblatex::Bibliography;
use rayon::prelude::*;

/// Read a bibliography and convert it to a vector of references
pub fn read_bibliography(bibliography: Bibliography) -> Vec<Reference> {
    bibliography
        .into_vec()
        .par_iter()
        .map(Reference::from)
        .collect()
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
