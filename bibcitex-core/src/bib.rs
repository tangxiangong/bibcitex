use crate::{Error, Result};
use biblatex::{Bibliography, Chunk, EntryType, PermissiveType, Person, Spanned};
use fs_err as fs;
use std::{ops::Range, path::Path};

/// Parse BibTeX database `.bib` file
pub fn parse(file_path: impl AsRef<Path>) -> Result<Bibliography> {
    let src = fs::read_to_string(file_path)?;
    Ok(Bibliography::parse(&src)?)
}

/// Wrap a `biblatex::Entry` into a `Reference`, with detailed fields.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Reference {
    /// key
    pub key: String,
    /// type
    pub type_: EntryType,
    /// author
    pub author: Option<Vec<Person>>,
    /// title
    pub title: Option<Vec<Chunk>>,
    /// journal
    pub journal: Option<Vec<Chunk>>,
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
}

impl From<&biblatex::Entry> for Reference {
    fn from(entry: &biblatex::Entry) -> Self {
        let key = entry.key.clone();
        let type_ = entry.entry_type.clone();
        let author = entry.author().ok();
        let title = entry.title().ok().map(|spanned_chunks| {
            spanned_chunks
                .iter()
                .map(|spanned_chunk| spanned_chunk.v.clone())
                .collect()
        });
        let journal = entry.journal().ok().map(|spanned_chunks| {
            spanned_chunks
                .iter()
                .map(|spanned_chunk| spanned_chunk.v.clone())
                .collect()
        });
        let year = parse_year(entry).ok();
        let full_journal = parse_optional_field(entry, "fjournal");
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
        let note = entry.note().ok().map(|chunks| {
            chunks
                .iter()
                .map(|spanned_chunk| spanned_chunk.v.clone())
                .collect()
        });
        let doi = entry.doi().ok();
        let mrclass = parse_optional_field(entry, "mrclass");
        Self {
            key,
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
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Article {
    pub key: String,
    pub author: Vec<Person>,
    pub title: Vec<Chunk>,
    pub journal: Vec<Chunk>,
    pub year: i32,
    pub full_journal: Option<String>,
    pub volume: Option<i64>,
    pub number: Option<String>,
    pub pages: Option<Range<u32>>,
    pub note: Option<Vec<Chunk>>,
    pub doi: Option<String>,
    pub mrclass: Option<String>,
}

impl TryFrom<Reference> for Article {
    type Error = Error;
    fn try_from(value: Reference) -> Result<Self> {
        Ok(Self {
            key: value.key,
            author: value.author.ok_or(Error::MissingFiled(
                "The field `author` is missing or not compatible!".to_string(),
            ))?,
            title: value.title.ok_or(Error::MissingFiled(
                "The field `title` is missing or not compatible!".to_string(),
            ))?,
            journal: value.journal.ok_or(Error::MissingFiled(
                "The field `journal` is missing or not compatible!".to_string(),
            ))?,
            year: value.year.ok_or(Error::MissingFiled(
                "The field `year` is missing or not compatible!".to_string(),
            ))?,
            full_journal: value.full_journal,
            volume: value.volume,
            number: value.number,
            pages: value.pages,
            note: value.note,
            doi: value.doi,
            mrclass: value.mrclass,
        })
    }
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
pub(crate) fn parse_optional_field(entry: &biblatex::Entry, field: &str) -> Option<String> {
    if let Some(chunks) = entry.get(field) {
        let chunk = chunks.first()?;
        if let Spanned {
            v: Chunk::Normal(chunk),
            ..
        } = chunk
        {
            Some(chunk.trim().to_owned())
        } else {
            None
        }
    } else {
        None
    }
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
        let entry = bib.get("MR2864664").unwrap();
        let article = Reference::from(entry);
        println!("{article:#?}");
    }
}
