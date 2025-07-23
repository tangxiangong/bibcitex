use crate::bib::Reference;
use rayon::prelude::*;

pub fn search_references(references: &[Reference], query: &str) -> Vec<Reference> {
    if references.len() > 100 {
        par_search_references(references, query)
    } else {
        seq_search_references(references, query)
    }
}

pub fn par_search_references(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.to_lowercase();
    references
        .par_iter()
        .filter(|&reference| check(reference, &query))
        .cloned()
        .collect()
}

pub fn seq_search_references(references: &[Reference], query: &str) -> Vec<Reference> {
    let query = query.to_lowercase();
    references
        .iter()
        .filter(|&reference| check(reference, &query))
        .cloned()
        .collect()
}

fn check_key(reference: &Reference, query: &str) -> bool {
    reference.cite_key.contains(query)
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
