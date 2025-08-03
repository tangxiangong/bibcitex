use crate::{Error, Result, utils::merge_chunks};
use biblatex::{Bibliography, Chunk, EntryType, PermissiveType, Person, Spanned};
use dioxus::prelude::Props;
use fs_err as fs;
use std::{ops::Range, path::Path};

/// Parse BibTeX database `.bib` file
pub fn parse(file_path: impl AsRef<Path>) -> Result<Bibliography> {
    let src = fs::read_to_string(file_path)?;
    Ok(Bibliography::parse(&src)?)
}

/// Wrap a `biblatex::Entry` into a `Reference`, with detailed fields.
#[derive(Debug, PartialEq, Eq, Clone, Props)]
pub struct Reference {
    /// key
    pub cite_key: String,
    /// parse result
    pub source: String,
    /// type
    pub type_: EntryType,
    /// author
    pub author: Option<Vec<String>>,
    /// title
    pub title: Option<Vec<Chunk>>,
    /// journal
    pub journal: Option<String>,
    /// year
    pub year: Option<i32>,
    /// full_journal
    pub full_journal: Option<String>,
    /// volume
    pub volume: Option<i64>,
    /// number
    pub number: Option<String>,
    /// pages
    pub pages: Option<Range<u32>>,
    /// note
    pub note: Option<Vec<Chunk>>,
    /// doi
    pub doi: Option<String>,
    /// mrclass
    pub mrclass: Option<String>,
    /// publisher
    pub publisher: Option<Vec<String>>,
    /// isbn
    pub isbn: Option<String>,
    /// series
    pub series: Option<String>,
    /// url
    pub url: Option<String>,
    /// pdf
    pub file: Option<String>,
    /// abstract
    pub abstract_: Option<Vec<Chunk>>,
    /// edition
    pub edition: Option<i64>,
    /// issue
    pub issue: Option<Vec<Chunk>>,
    /// book pages
    pub book_pages: Option<String>,
}

impl From<&biblatex::Entry> for Reference {
    fn from(entry: &biblatex::Entry) -> Self {
        let key = entry.key.clone();
        let source = entry
            .to_bibtex_string()
            .unwrap_or_else(|_| entry.to_biblatex_string());
        let type_ = entry.entry_type.clone();
        let author = entry
            .author()
            .ok()
            .map(|persons| persons.iter().map(get_name).collect());
        let title = entry
            .title()
            .ok()
            .map(|chunks| merge_chunks(chunks.to_owned()));
        let journal = entry.journal().ok().and_then(|chunks| {
            merge_chunks(chunks.to_owned())
                .first()
                .map(|chunk| chunk.get().to_string())
        });
        let year = parse_year(entry).ok();
        let full_journal = parse_optional_field(entry, "fjournal")
            .and_then(|chunk| chunk.first().map(|chunk| chunk.get().to_string()));
        let volume = match entry.volume().ok() {
            Some(PermissiveType::Typed(value)) => Some(value),
            _ => None,
        };
        let number = match entry.number().ok() {
            None => None,
            Some(chunks) => chunks.first().and_then(|chunk| {
                if let Spanned {
                    v: Chunk::Normal(chunk),
                    ..
                } = chunk
                {
                    Some(chunk.trim().to_owned())
                } else {
                    None
                }
            }),
        };
        let pages = match entry.pages().ok() {
            Some(PermissiveType::Typed(value)) => Some(value),
            _ => None,
        }
        .and_then(|pages| pages.first().cloned());
        let note = entry
            .note()
            .ok()
            .map(|chunks| merge_chunks(chunks.to_owned()));
        let doi = entry.doi().ok();
        let mrclass = parse_optional_field(entry, "mrclass")
            .and_then(|chunks| chunks.first().map(|chunk| chunk.get().to_string()));
        let publisher = entry.publisher().ok().and_then(|v| {
            v.into_iter()
                .map(|chunks| {
                    merge_chunks(chunks)
                        .first()
                        .map(|chunk| chunk.get().to_string())
                })
                .collect::<Option<Vec<_>>>()
        });
        let series = entry.series().ok().and_then(|chunks| {
            merge_chunks(chunks.to_owned())
                .first()
                .map(|chunk| chunk.get().to_string())
        });
        let isbn = entry.isbn().ok().and_then(|chunks| {
            merge_chunks(chunks.to_owned())
                .first()
                .map(|chunk| chunk.get().to_string())
        });
        let url = entry.url().ok();
        let file = entry.file().ok();
        let abstract_ = entry
            .abstract_()
            .ok()
            .map(|chunks| merge_chunks(chunks.to_owned()));
        let edition = match entry.edition().ok() {
            Some(PermissiveType::Typed(value)) => Some(value),
            _ => None,
        };
        let issue = entry
            .issue()
            .ok()
            .map(|chunks| merge_chunks(chunks.to_owned()));
        let book_pages = parse_optional_field(entry, "pages")
            .and_then(|chunk| chunk.first().map(|chunk| chunk.get().to_string()));
        Self {
            cite_key: key,
            source,
            type_,
            author,
            title,
            journal,
            year,
            volume,
            number,
            pages,
            note,
            doi,
            mrclass,
            full_journal,
            publisher,
            series,
            isbn,
            url,
            file,
            abstract_,
            edition,
            issue,
            book_pages,
        }
    }
}

fn get_name(person: &Person) -> String {
    format!("{} {}", person.given_name, person.name)
}

/// Parse the year field
pub(crate) fn parse_year(entry: &biblatex::Entry) -> Result<i32> {
    let year = if let Some(chunks) = entry.get("year") {
        let chunk = chunks.first();
        if chunk.is_none() {
            return Err(Error::MissingFiled(
                "The field `year` is missing!".to_string(),
            ));
        }
        if let Spanned {
            v: Chunk::Normal(chunk),
            ..
        } = chunk.unwrap()
        {
            chunk.trim().to_owned()
        } else {
            return Err(Error::FieldType(
                "The type of field `year` is not compatible!".to_string(),
            ));
        }
    } else {
        return Err(Error::MissingFiled(
            "The field `year` is missing!".to_string(),
        ));
    };
    match year.parse::<i32>() {
        Ok(year) => Ok(year),
        Err(e) => Err(Error::FieldType(format!(
            "The type of field `year` is not compatible! {e}",
        ))),
    }
}

/// Parse optional field
pub(crate) fn parse_optional_field(entry: &biblatex::Entry, field: &str) -> Option<Vec<Chunk>> {
    entry
        .get(field)
        .map(|chunks| merge_chunks(chunks.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse() {
        let path = Path::new("../database.bib");
        let bib = parse(path).unwrap();
        let entry = bib.get("MR4293957").unwrap();
        let title = entry.title().unwrap();
        println!("{title:#?}");
    }

    #[test]
    fn test_show_article() {
        let path = Path::new("../database.bib");
        let bib = parse(path).unwrap();
        let entry = bib.get("MR0404849").unwrap();
        let article = Reference::from(entry);
        println!("{article:#?}");
    }
}
