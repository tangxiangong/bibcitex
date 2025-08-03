use crate::bib::Reference;
use biblatex::EntryType;

/// Filters references to articles.
pub fn filter_article(references: Vec<Reference>) -> Vec<Reference> {
    references
        .into_iter()
        .filter(|r| r.type_ == EntryType::Article)
        .collect()
}

/// Filters references to books.
pub fn filter_book(references: Vec<Reference>) -> Vec<Reference> {
    references
        .into_iter()
        .filter(|r| r.type_ == EntryType::Book)
        .collect()
}

/// Filters references to theses.
pub fn filter_thesis(references: Vec<Reference>) -> Vec<Reference> {
    references
        .into_iter()
        .filter(|r| {
            r.type_ == EntryType::PhdThesis
                || r.type_ == EntryType::MastersThesis
                || r.type_ == EntryType::Thesis
        })
        .collect()
}
