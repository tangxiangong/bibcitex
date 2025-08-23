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

/// Filters references to booklet.
pub fn filter_booklet(references: Vec<Reference>) -> Vec<Reference> {
    references
        .into_iter()
        .filter(|r| r.type_ == EntryType::Booklet)
        .collect()
}

/// Filters references to inbook.
pub fn filter_inbook(references: Vec<Reference>) -> Vec<Reference> {
    references
        .into_iter()
        .filter(|r| r.type_ == EntryType::InBook)
        .collect()
}

/// Filters references to incollection.
pub fn filter_incollection(references: Vec<Reference>) -> Vec<Reference> {
    references
        .into_iter()
        .filter(|r| r.type_ == EntryType::InCollection)
        .collect()
}

/// Filters references to inproceedings.
pub fn filter_inproceedings(references: Vec<Reference>) -> Vec<Reference> {
    references
        .into_iter()
        .filter(|r| r.type_ == EntryType::InProceedings)
        .collect()
}

/// Filters references to misc.
pub fn filter_misc(references: Vec<Reference>) -> Vec<Reference> {
    references
        .into_iter()
        .filter(|r| r.type_ == EntryType::Misc)
        .collect()
}

/// Filters references to techreport.
pub fn filter_techreport(references: Vec<Reference>) -> Vec<Reference> {
    references
        .into_iter()
        .filter(|r| r.type_ == EntryType::TechReport)
        .collect()
}
