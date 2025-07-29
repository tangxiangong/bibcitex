use crate::bib::Reference;
use rayon::prelude::*;

const THRESHOLD_PARALLEL_SIZE: usize = 100;

/// Search for references that match the given query.
pub fn search_references(references: &[Reference], query: &str) -> Vec<Reference> {
    if references.len() > THRESHOLD_PARALLEL_SIZE {
        par_search_references(references, query)
    } else {
        seq_search_references(references, query)
    }
}

/// Search for references that match the given query in parallel.
pub fn par_search_references(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.trim().to_lowercase();
    references
        .par_iter()
        .filter(|&reference| check(reference, &query))
        .cloned()
        .collect()
}

/// Search for references that match the given query sequentially.
pub fn seq_search_references(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.trim().to_lowercase();
    references
        .iter()
        .filter(|&reference| check(reference, &query))
        .cloned()
        .collect()
}

fn check_key(reference: &Reference, query: &str) -> bool {
    reference.cite_key.to_lowercase().contains(query)
}

fn check_title(reference: &Reference, query: &str) -> bool {
    if let Some(title) = &reference.title {
        for chunk in title {
            if chunk.get().to_lowercase().contains(query) {
                return true;
            }
        }
    }
    false
}

fn check_author(reference: &Reference, query: &str) -> bool {
    if let Some(author) = &reference.author {
        for name in author {
            if name.to_lowercase().contains(query) {
                return true;
            }
        }
    }
    false
}

fn check_journal(reference: &Reference, query: &str) -> bool {
    if let Some(journal) = &reference.journal {
        journal.to_lowercase().contains(query)
    } else {
        false
    }
}

fn check_full_journal(reference: &Reference, query: &str) -> bool {
    if let Some(journal) = &reference.full_journal {
        journal.to_lowercase().contains(query)
    } else {
        false
    }
}

fn check_year(reference: &Reference, query: &str) -> bool {
    if let Some(year) = &reference.year {
        year.to_string().eq(query)
    } else {
        false
    }
}

fn check_note(reference: &Reference, query: &str) -> bool {
    if let Some(note) = &reference.note {
        for chunk in note {
            if chunk.get().to_lowercase().contains(query) {
                return true;
            }
        }
    }
    false
}

fn check(reference: &Reference, query: &str) -> bool {
    check_key(reference, query)
        || check_title(reference, query)
        || check_author(reference, query)
        || check_journal(reference, query)
        || check_full_journal(reference, query)
        || check_year(reference, query)
        || check_note(reference, query)
}

/// Search for references that match the given author sequentially.
pub fn seq_search_references_by_author(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.trim().to_lowercase();
    references
        .iter()
        .filter(|reference| check_author(reference, &query))
        .cloned()
        .collect()
}

/// Search for references that match the given author in parallel.
pub fn par_search_references_by_author(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.trim().to_lowercase();
    references
        .par_iter()
        .filter(|reference| check_author(reference, &query))
        .cloned()
        .collect()
}

/// Search for references that match the given author sequentially or in parallel.
pub fn search_references_by_author(references: &[Reference], query: &str) -> Vec<Reference> {
    if references.len() > THRESHOLD_PARALLEL_SIZE {
        par_search_references_by_author(references, query)
    } else {
        seq_search_references_by_author(references, query)
    }
}

/// Search for references that match the given title sequentially.
pub fn seq_search_references_by_title(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.trim().to_lowercase();
    references
        .iter()
        .filter(|reference| check_title(reference, &query))
        .cloned()
        .collect()
}

/// Search for references that match the given title in parallel.
pub fn par_search_references_by_title(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.trim().to_lowercase();
    references
        .par_iter()
        .filter(|reference| check_title(reference, &query))
        .cloned()
        .collect()
}

/// Search for references that match the given title sequentially or in parallel.
pub fn search_references_by_title(references: &[Reference], query: &str) -> Vec<Reference> {
    if references.len() > THRESHOLD_PARALLEL_SIZE {
        par_search_references_by_title(references, query)
    } else {
        seq_search_references_by_title(references, query)
    }
}

/// Search for references that match the given journal sequentially.
pub fn seq_search_references_by_journal(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.trim().to_lowercase();
    references
        .iter()
        .filter(|reference| check_journal(reference, &query))
        .cloned()
        .collect()
}

/// Search for references that match the given journal in parallel.
pub fn par_search_references_by_journal(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.trim().to_lowercase();
    references
        .par_iter()
        .filter(|reference| check_journal(reference, &query))
        .cloned()
        .collect()
}

/// Search for references that match the given journal sequentially or in parallel.
pub fn search_references_by_journal(references: &[Reference], query: &str) -> Vec<Reference> {
    if references.len() > THRESHOLD_PARALLEL_SIZE {
        par_search_references_by_journal(references, query)
    } else {
        seq_search_references_by_journal(references, query)
    }
}

/// Search for references that match the given year sequentially.
pub fn seq_search_references_by_year(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.trim().to_lowercase();
    references
        .iter()
        .filter(|reference| check_year(reference, &query))
        .cloned()
        .collect()
}

/// Search for references that match the given year in parallel.
pub fn par_search_references_by_year(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.trim().to_lowercase();
    references
        .par_iter()
        .filter(|reference| check_year(reference, &query))
        .cloned()
        .collect()
}

/// Search for references that match the given year sequentially or in parallel.
pub fn search_references_by_year(references: &[Reference], query: &str) -> Vec<Reference> {
    if references.len() > THRESHOLD_PARALLEL_SIZE {
        par_search_references_by_year(references, query)
    } else {
        seq_search_references_by_year(references, query)
    }
}
