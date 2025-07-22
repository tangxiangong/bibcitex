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
